<?php

namespace App\Http\Requests;

use App\Rules\StellarAccountId;
use Illuminate\Foundation\Http\FormRequest;

class ShowMemberRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true; // consulta pública (qualquer passaporte)
    }

    protected function prepareForValidation(): void
    {
        $this->merge(['address' => $this->route('address')]);
    }

    /**
     * @return array<string, array<int, mixed>>
     */
    public function rules(): array
    {
        return [
            'address' => ['required', 'string', new StellarAccountId],
        ];
    }
}
