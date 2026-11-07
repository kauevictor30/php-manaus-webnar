#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events as _, AuthorizedFunction, AuthorizedInvocation, MockAuth, MockAuthInvoke},
    Address, Env, Event, IntoVal, Symbol,
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
