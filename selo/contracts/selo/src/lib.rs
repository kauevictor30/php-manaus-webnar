#![no_std]
//! Selo — badges soulbound (intransferíveis) que formam a reputação on-chain.
//! Não existe transferência e o contrato não implementa SEP-41 (RN-02).

use soroban_sdk::{contract, contracterror, contractevent, contracttype, Address};

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

#[contract]
pub struct Selo;

mod test;
