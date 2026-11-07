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
    let badges = client.badges_of(&membro);
    assert_eq!(badges.len(), 1);
    assert_eq!(badges.get_unchecked(0).issuer, emissor);
    assert_eq!(client.reputation(&membro), 1);
}

// ------------------------------------------------------------ revoke / reputation / badges_of (T06)

fn badge(kind: BadgeKind, event_id: u32, issuer: &Address, issued_at: u64) -> Badge {
    Badge {
        kind,
        event_id,
        issuer: issuer.clone(),
        issued_at,
    }
}

#[test]
fn reputation_e_zero_para_quem_nao_tem_badge() {
    let (env, _id, client) = novo();
    let ninguem = Address::generate(&env);
    assert_eq!(client.reputation(&ninguem), 0);
}

#[test]
fn badges_of_e_vazio_para_quem_nao_tem_badge() {
    let (env, _id, client) = novo();
    let ninguem = Address::generate(&env);
    assert_eq!(client.badges_of(&ninguem), Vec::<Badge>::new(&env));
}

#[test]
fn reputation_e_badges_of_nao_exigem_auth() {
    let (env, _id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &1);
    env.set_auths(&[]); // desliga o mock_all_auths
    assert_eq!(client.try_reputation(&membro), Ok(Ok(3)));
    assert!(client.try_badges_of(&membro).is_ok());
    assert!(env.auths().is_empty());
}

#[test]
fn badges_of_retorna_na_ordem_de_emissao() {
    let (env, _id, client, _admin, emissor) = com_emissor();
    env.ledger().set_timestamp(100);
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Participante, &1);
    client.issue(&emissor, &membro, &BadgeKind::Palestrante, &2);
    assert_eq!(
        client.badges_of(&membro),
        vec![
            &env,
            badge(BadgeKind::Participante, 1, &emissor, 100),
            badge(BadgeKind::Palestrante, 2, &emissor, 100),
        ]
    );
    assert_eq!(client.reputation(&membro), 6);
}

#[test]
fn emitir_3_e_revogar_o_do_meio() {
    let (env, id, client, _admin, emissor) = com_emissor();
    env.ledger().set_timestamp(500);
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Participante, &1); // 1
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &2); // 3
    client.issue(&emissor, &membro, &BadgeKind::Palestrante, &3); // 5
    assert_eq!(client.reputation(&membro), 9);

    client.revoke(&membro, &BadgeKind::Mentor, &2);

    assert_eq!(
        client.badges_of(&membro),
        vec![
            &env,
            badge(BadgeKind::Participante, 1, &emissor, 500),
            badge(BadgeKind::Palestrante, 3, &emissor, 500),
        ]
    );
    assert_eq!(client.reputation(&membro), 6);
    assert_eq!(badge_gravado(&env, &id, &membro, BadgeKind::Mentor, 2), None);
    assert_eq!(
        lista_gravada(&env, &id, &membro),
        Some(vec![
            &env,
            (BadgeKind::Participante, 1u32),
            (BadgeKind::Palestrante, 3u32)
        ])
    );
}

#[test]
fn revogar_o_primeiro_e_depois_o_ultimo_remove_pelos_valores() {
    let (env, _id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Participante, &1);
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &2);
    client.issue(&emissor, &membro, &BadgeKind::Organizador, &3);
    client.issue(&emissor, &membro, &BadgeKind::Palestrante, &4);

    client.revoke(&membro, &BadgeKind::Participante, &1);
    client.revoke(&membro, &BadgeKind::Palestrante, &4);

    let restantes: std::vec::Vec<(BadgeKind, u32)> = client
        .badges_of(&membro)
        .iter()
        .map(|b| (b.kind, b.event_id))
        .collect();
    assert_eq!(
        restantes,
        std::vec![(BadgeKind::Mentor, 2), (BadgeKind::Organizador, 3)]
    );
    assert_eq!(client.reputation(&membro), 6);
}

#[test]
fn revoke_exige_auth_do_admin() {
    let (env, id, client, admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &2);
    client.revoke(&membro, &BadgeKind::Mentor, &2);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    id.clone(),
                    Symbol::new(&env, "revoke"),
                    (membro.clone(), BadgeKind::Mentor, 2u32).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );
}

#[test]
fn revoke_emite_evento_revoked() {
    let (env, id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Organizador, &8);
    client.revoke(&membro, &BadgeKind::Organizador, &8);
    assert_eq!(
        env.events().all(),
        [RevokedEvent {
            member: membro.clone(),
            kind: BadgeKind::Organizador,
            event_id: 8,
        }
        .to_xdr(&env, &id)]
    );
}

#[test]
fn revoke_estende_ttl_da_lista_e_da_reputacao() {
    let (env, id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Participante, &1);
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &2);
    // avança o ledger até o TTL ficar abaixo do TTL_THRESHOLD (extend_ttl só age aí)
    env.ledger()
        .with_mut(|l| l.sequence_number += TTL_EXTEND_TO - TTL_THRESHOLD + 1);
    let lista = DataKey::BadgeList(membro.clone());
    let rep = DataKey::Reputation(membro.clone());
    assert_eq!(ttl_persistent(&env, &id, &lista), TTL_THRESHOLD - 1);
    assert_eq!(ttl_persistent(&env, &id, &rep), TTL_THRESHOLD - 1);

    client.revoke(&membro, &BadgeKind::Participante, &1);

    assert_eq!(ttl_persistent(&env, &id, &lista), TTL_EXTEND_TO);
    assert_eq!(ttl_persistent(&env, &id, &rep), TTL_EXTEND_TO);
}

#[test]
fn revoke_sem_auth_falha() {
    let (env, id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &2);
    env.set_auths(&[]); // desliga o mock_all_auths
    assert!(matches!(
        client.try_revoke(&membro, &BadgeKind::Mentor, &2),
        Err(Err(_))
    ));
    assert!(badge_gravado(&env, &id, &membro, BadgeKind::Mentor, 2).is_some());
    assert_eq!(client.reputation(&membro), 3);
}

#[test]
fn revoke_assinado_pelo_emissor_falha() {
    let (env, id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &2);
    env.mock_auths(&[MockAuth {
        address: &emissor,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "revoke",
            args: (membro.clone(), BadgeKind::Mentor, 2u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(matches!(
        client.try_revoke(&membro, &BadgeKind::Mentor, &2),
        Err(Err(_))
    ));
    assert_eq!(client.reputation(&membro), 3);
}

#[test]
fn revoke_inexistente_falha_badge_not_found() {
    let (env, _id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    assert_eq!(
        client.try_revoke(&membro, &BadgeKind::Mentor, &2),
        Err(Ok(Error::BadgeNotFound))
    );
    // membro com outros badges, mas não esse
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &3);
    client.issue(&emissor, &membro, &BadgeKind::Participante, &2);
    assert_eq!(
        client.try_revoke(&membro, &BadgeKind::Mentor, &2),
        Err(Ok(Error::BadgeNotFound))
    );
    assert_eq!(client.reputation(&membro), 4);
}

#[test]
fn revoke_duas_vezes_falha_badge_not_found() {
    let (env, _id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Mentor, &2);
    client.revoke(&membro, &BadgeKind::Mentor, &2);
    assert_eq!(
        client.try_revoke(&membro, &BadgeKind::Mentor, &2),
        Err(Ok(Error::BadgeNotFound))
    );
    assert_eq!(client.reputation(&membro), 0);
}

#[test]
fn revoke_antes_do_init_falha_not_initialized() {
    let (env, _id, client) = novo();
    env.mock_all_auths();
    let membro = Address::generate(&env);
    assert_eq!(
        client.try_revoke(&membro, &BadgeKind::Mentor, &2),
        Err(Ok(Error::NotInitialized))
    );
}

#[test]
fn revogar_e_reemitir_funciona() {
    let (env, _id, client, _admin, emissor) = com_emissor();
    let membro = Address::generate(&env);
    client.issue(&emissor, &membro, &BadgeKind::Palestrante, &4);
    client.issue(&emissor, &membro, &BadgeKind::Participante, &4);
    client.revoke(&membro, &BadgeKind::Palestrante, &4);
    assert_eq!(client.reputation(&membro), 1);

    assert_eq!(
        client.try_issue(&emissor, &membro, &BadgeKind::Palestrante, &4),
        Ok(Ok(()))
    );
    assert_eq!(client.reputation(&membro), 6);
    let ordem: std::vec::Vec<(BadgeKind, u32)> = client
        .badges_of(&membro)
        .iter()
        .map(|b| (b.kind, b.event_id))
        .collect();
    assert_eq!(
        ordem,
        std::vec![(BadgeKind::Participante, 4), (BadgeKind::Palestrante, 4)]
    );
}
