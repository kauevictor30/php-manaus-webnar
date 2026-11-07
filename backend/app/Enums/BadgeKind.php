<?php

namespace App\Enums;

/**
 * Espelho do enum BadgeKind do contrato Selo (design §2.1).
 * Os pesos DEVEM ser idênticos ao weight() do contrato (D-06).
 */
enum BadgeKind: string
{
    case Participante = 'Participante';
    case Mentor = 'Mentor';
    case Organizador = 'Organizador';
    case Palestrante = 'Palestrante';

    public function weight(): int
    {
        return match ($this) {
            self::Participante => 1,
            self::Mentor => 3,
            self::Organizador => 3,
            self::Palestrante => 5,
        };
    }
}
