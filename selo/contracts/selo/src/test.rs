#![cfg(test)]

use super::*;

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
