#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{
        storage::{Instance as _, Persistent as _},
        Address as _, AuthorizedFunction, AuthorizedInvocation, Events as _, Ledger as _,
        MockAuth, MockAuthInvoke,
    },
    vec, Address, Env, Event, IntoVal, Symbol, Vec,
};

// ------------------------------------------------------------ helpers

fn novo() -> (Env, Address, SeloClient<'static>) {
    let env = Env::default();
    let id = env.register(Selo, ());
    let client = SeloClient::new(&env, &id);
    (env, id, client)
}

/// Contrato inicializado com um admin, com todas as auths simuladas.
fn iniciado() -> (Env, Address, SeloClient<'static>, Address) {
    let (env, id, client) = novo();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    client.init(&admin);
    (env, id, client, admin)
}

fn admin_gravado(env: &Env, id: &Address) -> Option<Address> {
    env.as_contract(id, || env.storage().instance().get(&DataKey::Admin))
}

fn issuer_gravado(env: &Env, id: &Address, issuer: &Address) -> Option<bool> {
    env.as_contract(id, || {
        env.storage()
            .persistent()
            .get(&DataKey::Issuer(issuer.clone()))
    })
}

/// Contrato inicializado + um emissor credenciado.
fn com_emissor() -> (Env, Address, SeloClient<'static>, Address, Address) {
    let (env, id, client, admin) = iniciado();
    let emissor = Address::generate(&env);
    client.set_issuer(&emissor, &true);
    (env, id, client, admin, emissor)
}

fn badge_gravado(
    env: &Env,
    id: &Address,
    member: &Address,
    kind: BadgeKind,
    event_id: u32,
) -> Option<Badge> {
    env.as_contract(id, || {
        env.storage()
            .persistent()
            .get(&DataKey::Badge(member.clone(), kind, event_id))
    })
}

fn lista_gravada(env: &Env, id: &Address, member: &Address) -> Option<Vec<(BadgeKind, u32)>> {
    env.as_contract(id, || {
        env.storage()
            .persistent()
            .get(&DataKey::BadgeList(member.clone()))
    })
}

fn reputacao_gravada(env: &Env, id: &Address, member: &Address) -> Option<u32> {
    env.as_contract(id, || {
        env.storage()
            .persistent()
            .get(&DataKey::Reputation(member.clone()))
    })
}

fn ttl_persistent(env: &Env, id: &Address, key: &DataKey) -> u32 {
    env.as_contract(id, || env.storage().persistent().get_ttl(key))
}

// ------------------------------------------------------------ weight (T03)

#[test]
fn weight_participante_vale_1() {
    assert_eq!(weight(BadgeKind::Participante), 1);
}

#[test]
fn weight_mentor_vale_3() {
    assert_eq!(weight(BadgeKind::Mentor), 3);
}

#[test]
fn weight_organizador_vale_3() {
    assert_eq!(weight(BadgeKind::Organizador), 3);
}

#[test]
fn weight_palestrante_vale_5() {
    assert_eq!(weight(BadgeKind::Palestrante), 5);
}

// ------------------------------------------------------------ init (T04)

#[test]
fn init_grava_admin() {
    let (env, id, client) = novo();
    let admin = Address::generate(&env);
    client.init(&admin);
    assert_eq!(admin_gravado(&env, &id), Some(admin));
}

#[test]
fn init_nao_exige_auth() {
    // init não tem auth (design §2.5); nenhuma auth é simulada aqui.
    let (env, _id, client) = novo();
    let admin = Address::generate(&env);
    assert_eq!(client.try_init(&admin), Ok(Ok(())));
    assert!(env.auths().is_empty());
}

#[test]
fn init_duas_vezes_falha_already_initialized() {
    let (env, id, client, admin) = iniciado();
    let outro = Address::generate(&env);
    assert_eq!(
        client.try_init(&outro),
        Err(Ok(Error::AlreadyInitialized))
    );
    // o admin original continua gravado
    assert_eq!(admin_gravado(&env, &id), Some(admin));
}

// ------------------------------------------------------------ set_issuer (T04)

#[test]
fn set_issuer_ativa_e_exige_auth_do_admin() {
    let (env, id, client, admin) = iniciado();
    let emissor = Address::generate(&env);
    client.set_issuer(&emissor, &true);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    id.clone(),
                    Symbol::new(&env, "set_issuer"),
                    (emissor.clone(), true).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );
    assert_eq!(issuer_gravado(&env, &id, &emissor), Some(true));
}

#[test]
fn set_issuer_desativa() {
    let (env, id, client, _admin) = iniciado();
    let emissor = Address::generate(&env);
    client.set_issuer(&emissor, &true);
    client.set_issuer(&emissor, &false);
    assert_eq!(issuer_gravado(&env, &id, &emissor), Some(false));
}

#[test]
fn set_issuer_emite_evento_issuer() {
    let (env, id, client, _admin) = iniciado();
    let emissor = Address::generate(&env);
    client.set_issuer(&emissor, &true);
    assert_eq!(
        env.events().all(),
        [IssuerEvent {
            issuer: emissor.clone(),
            enabled: true
        }
        .to_xdr(&env, &id)]
    );
}

#[test]
fn set_issuer_sem_auth_falha() {
    let (env, id, client) = novo();
    let admin = Address::generate(&env);
    client.init(&admin); // init não precisa de auth
    let emissor = Address::generate(&env);
    // nenhuma auth simulada: falha na auth do host, não com erro do contrato
    assert!(matches!(
        client.try_set_issuer(&emissor, &true),
        Err(Err(_))
    ));
    assert_eq!(issuer_gravado(&env, &id, &emissor), None);
}

#[test]
fn set_issuer_assinado_por_outro_falha() {
    let (env, id, client) = novo();
    let admin = Address::generate(&env);
    client.init(&admin);
    let intruso = Address::generate(&env);
    let emissor = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &intruso,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "set_issuer",
            args: (emissor.clone(), true).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(matches!(
        client.try_set_issuer(&emissor, &true),
        Err(Err(_))
    ));
    assert_eq!(issuer_gravado(&env, &id, &emissor), None);
}

#[test]
fn set_issuer_antes_do_init_falha_not_initialized() {
    let (env, _id, client) = novo();
    env.mock_all_auths();
    let emissor = Address::generate(&env);
    assert_eq!(
        client.try_set_issuer(&emissor, &true),
        Err(Ok(Error::NotInitialized))
    );
}

// ------------------------------------------------------------ issue (T05)

#[test]
fn issue_grava_badge_lista_e_reputacao() {
    let (env, id, client, _admin, emissor) = com_emissor();
    env.ledger().set_timestamp(1_762_000_000);
    let membro = Address::generate(&env);

    client.issue(&emissor, &membro, &BadgeKind::Mentor, &42);

    assert_eq!(
        badge_gravado(&env, &id, &membro, BadgeKind::Mentor, 42),
        Some(Badge {
            kind: BadgeKind::Mentor,
            event_id: 42,
            issuer: emissor.clone(),
            issued_at: 1_762_000_000,
        })
    );
    assert_eq!(
        lista_gravada(&env, &id, &membro),
        Some(vec![&env, (BadgeKind::Mentor, 42u32)])
    );
    assert_eq!(reputacao_gravada(&env, &id, &membro), Some(3));
}

#[test]
fn issue_exige_auth_so_do_emissor_e_nao_do_membro() {
    let (env, id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Participante, &1);

    assert_eq!(
        env.auths(),
        std::vec![(
            emissor.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    id.clone(),
                    Symbol::new(&env, "issue"),
                    (emissor.clone(), membro.clone(), BadgeKind::Participante, 1u32)
                        .into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );
}

#[test]
fn issue_estende_ttl_das_3_chaves() {
    let (env, id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Palestrante, &7);

    for key in [
        DataKey::Badge(membro.clone(), BadgeKind::Palestrante, 7),
        DataKey::BadgeList(membro.clone()),
        DataKey::Reputation(membro.clone()),
    ] {
        assert_eq!(ttl_persistent(&env, &id, &key), TTL_EXTEND_TO);
    }
    assert_eq!(
        env.as_contract(&id, || env.storage().instance().get_ttl()),
        TTL_EXTEND_TO
    );
}

#[test]
fn issue_emite_evento_issued() {
    let (env, id, client, _admin, emissor) = com_emissor();
    env.ledger().set_timestamp(1_762_000_123);
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Organizador, &9);

    assert_eq!(
        env.events().all(),
        [IssuedEvent {
            member: membro.clone(),
            kind: BadgeKind::Organizador,
            event_id: 9,
            issuer: emissor.clone(),
            issued_at: 1_762_000_123,
        }
        .to_xdr(&env, &id)]
    );
}

#[test]
fn issue_sem_auth_falha() {
    let (env, id, client) = novo();
    let admin = Address::generate(&env);
    client.init(&admin);
    let emissor = Address::generate(&env);
    // credencia o emissor só com a auth do admin
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "set_issuer",
            args: (emissor.clone(), true).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.set_issuer(&emissor, &true);

    let membro = Address::generate(&env);
    assert!(matches!(
        client.try_issue(&emissor, &membro, &BadgeKind::Participante, &1),
        Err(Err(_))
    ));
    assert_eq!(
        badge_gravado(&env, &id, &membro, BadgeKind::Participante, 1),
        None
    );
}

#[test]
fn issue_assinado_pelo_membro_em_vez_do_emissor_falha() {
    let (env, id, client) = novo();
    let admin = Address::generate(&env);
    client.init(&admin);
    let emissor = Address::generate(&env);
    let membro = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "set_issuer",
            args: (emissor.clone(), true).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.set_issuer(&emissor, &true);

    env.mock_auths(&[MockAuth {
        address: &membro,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "issue",
            args: (emissor.clone(), membro.clone(), BadgeKind::Participante, 1u32)
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(matches!(
        client.try_issue(&emissor, &membro, &BadgeKind::Participante, &1),
        Err(Err(_))
    ));
}

#[test]
fn issue_por_nao_emissor_falha_not_issuer() {
    let (env, _id, client, _admin) = iniciado();
    let qualquer = Address::generate(&env);
    let membro = Address::generate(&env);
    assert_eq!(
        client.try_issue(&qualquer, &membro, &BadgeKind::Participante, &1),
        Err(Ok(Error::NotIssuer))
    );
}

#[test]
fn issue_para_si_mesmo_falha_self_issue() {
    let (env, id, client, _admin, emissor) = com_emissor();
    assert_eq!(
        client.try_issue(&emissor, &emissor, &BadgeKind::Palestrante, &1),
        Err(Ok(Error::SelfIssue))
    );
    assert_eq!(reputacao_gravada(&env, &id, &emissor), None);
}

#[test]
fn issue_duplicado_falha_already_has_badge() {
    let (env, id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &5);
    assert_eq!(
        client.try_issue(&emissor, &membro, &BadgeKind::Mentor, &5),
        Err(Ok(Error::AlreadyHasBadge))
    );
    // nada mudou
    assert_eq!(reputacao_gravada(&env, &id, &membro), Some(3));
    assert_eq!(
        lista_gravada(&env, &id, &membro),
        Some(vec![&env, (BadgeKind::Mentor, 5u32)])
    );
}

#[test]
fn issue_mesmo_tipo_em_outro_evento_e_outro_tipo_no_mesmo_evento_passam() {
    let (env, id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &5);
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &6);
    client.issue(&emissor, &membro, &BadgeKind::Participante, &5);
    assert_eq!(reputacao_gravada(&env, &id, &membro), Some(3 + 3 + 1));
}

#[test]
fn issue_emissor_desativado_falha_not_issuer_e_badges_antigos_continuam() {
    let (env, id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Participante, &1);

    client.set_issuer(&emissor, &false);

    let outro = Address::generate(&env);
    assert_eq!(
        client.try_issue(&emissor, &outro, &BadgeKind::Participante, &1),
        Err(Ok(Error::NotIssuer))
    );
    // RN-07: o badge antigo continua válido
    assert!(badge_gravado(&env, &id, &membro, BadgeKind::Participante, 1).is_some());
    assert_eq!(reputacao_gravada(&env, &id, &membro), Some(1));
}
