#![no_std]
//! Selo — badges soulbound (intransferíveis) que formam a reputação on-chain.
//! Não existe transferência e o contrato não implementa SEP-41 (RN-02).

use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Env, Vec,
};

// ---------------------------------------------------------------- 2.1 Tipos

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BadgeKind {
    Participante,
    Mentor,
    Organizador,
    Palestrante,
}

/// Pesos da reputação (RN-05). Duplicados no Laravel (D-06).
pub fn weight(kind: BadgeKind) -> u32 {
    match kind {
        BadgeKind::Participante => 1,
        BadgeKind::Mentor => 3,
        BadgeKind::Organizador => 3,
        BadgeKind::Palestrante => 5,
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Badge {
    pub kind: BadgeKind,
    pub event_id: u32,
    pub issuer: Address,
    pub issued_at: u64,
}

// ---------------------------------------------------------------- 2.2 Storage

#[contracttype]
pub enum DataKey {
    Admin,                          // instance   -> Address
    Issuer(Address),                // persistent -> bool
    Badge(Address, BadgeKind, u32), // persistent -> Badge (chave de dedup — RN-03)
    BadgeList(Address),             // persistent -> Vec<(BadgeKind, u32)>
    Reputation(Address),            // persistent -> u32
}

/// ~30 dias de ledgers (5s por ledger).
pub const TTL_THRESHOLD: u32 = 17_280 * 30;
/// ~120 dias de ledgers.
pub const TTL_EXTEND_TO: u32 = 17_280 * 120;

// ---------------------------------------------------------------- 2.3 Erros

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    /// Reservado (D-04): a falta de assinatura do admin falha na auth do host.
    NotAdmin = 3,
    NotIssuer = 4,
    SelfIssue = 5,
    AlreadyHasBadge = 6,
    BadgeNotFound = 7,
}

// ---------------------------------------------------------------- 2.4 Eventos

#[contractevent(topics = ["issuer"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuerEvent {
    #[topic]
    pub issuer: Address,
    pub enabled: bool,
}

#[contractevent(topics = ["issued"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuedEvent {
    #[topic]
    pub member: Address,
    pub kind: BadgeKind,
    pub event_id: u32,
    pub issuer: Address,
    pub issued_at: u64,
}

#[contractevent(topics = ["revoked"])]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevokedEvent {
    #[topic]
    pub member: Address,
    pub kind: BadgeKind,
    pub event_id: u32,
}

// ---------------------------------------------------------------- 2.5 Contrato

fn bump_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn bump_persistent(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

fn read_admin(env: &Env) -> Result<Address, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)
}

#[contract]
pub struct Selo;

#[contractimpl]
impl Selo {
    /// Grava o admin. Só pode ser chamado uma vez (RN-01).
    pub fn init(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        bump_instance(&env);
        Ok(())
    }

    /// Credencia (`true`) ou descredencia (`false`) um emissor. Só o admin.
    pub fn set_issuer(env: Env, issuer: Address, enabled: bool) -> Result<(), Error> {
        let admin = read_admin(&env)?;
        admin.require_auth();

        let key = DataKey::Issuer(issuer.clone());
        env.storage().persistent().set(&key, &enabled);
        bump_persistent(&env, &key);
        bump_instance(&env);

        IssuerEvent { issuer, enabled }.publish(&env);
        Ok(())
    }

    /// Emite (carimba) um badge. Exige a assinatura do emissor; o membro não assina.
    pub fn issue(
        env: Env,
        issuer: Address,
        member: Address,
        kind: BadgeKind,
        event_id: u32,
    ) -> Result<(), Error> {
        issuer.require_auth();

        // 1) emissor credenciado e ativo (RN-07)
        let active: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Issuer(issuer.clone()))
            .unwrap_or(false);
        if !active {
            return Err(Error::NotIssuer);
        }
        // 2) sem auto-emissão (RN-04)
        if issuer == member {
            return Err(Error::SelfIssue);
        }
        // 3) dedup (RN-03)
        let badge_key = DataKey::Badge(member.clone(), kind, event_id);
        if env.storage().persistent().has(&badge_key) {
            return Err(Error::AlreadyHasBadge);
        }

        let issued_at = env.ledger().timestamp();
        let badge = Badge {
            kind,
            event_id,
            issuer: issuer.clone(),
            issued_at,
        };
        env.storage().persistent().set(&badge_key, &badge);
        bump_persistent(&env, &badge_key);

        let list_key = DataKey::BadgeList(member.clone());
        let mut list: Vec<(BadgeKind, u32)> = env
            .storage()
            .persistent()
            .get(&list_key)
            .unwrap_or_else(|| Vec::new(&env));
        list.push_back((kind, event_id));
        env.storage().persistent().set(&list_key, &list);
        bump_persistent(&env, &list_key);

        let rep_key = DataKey::Reputation(member.clone());
        let rep: u32 = env.storage().persistent().get(&rep_key).unwrap_or(0);
        let rep = rep
            .checked_add(weight(kind))
            .expect("reputation overflow");
        env.storage().persistent().set(&rep_key, &rep);
        bump_persistent(&env, &rep_key);

        bump_instance(&env);

        IssuedEvent {
            member,
            kind,
            event_id,
            issuer,
            issued_at,
        }
        .publish(&env);
        Ok(())
    }
}

mod test;
