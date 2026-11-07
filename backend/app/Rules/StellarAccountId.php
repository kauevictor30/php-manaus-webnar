<?php

namespace App\Rules;

use Closure;
use Illuminate\Contracts\Validation\ValidationRule;
use Soneso\StellarSDK\Crypto\StrKey;

/** Endereço de conta Stellar G... (56 caracteres, checksum válido via StrKey). */
class StellarAccountId implements ValidationRule
{
    public function validate(string $attribute, mixed $value, Closure $fail): void
    {
        if (! is_string($value) || strlen($value) !== 56 || $value[0] !== 'G' || ! StrKey::isValidAccountId($value)) {
            $fail('O campo :attribute deve ser um endereço Stellar G... válido.');
        }
    }
}
