<?php

namespace App\Http\Resources;

use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/** @mixin \App\Models\BadgeRecord */
class BadgeRecordResource extends JsonResource
{
    /**
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'kind' => $this->kind->value,
            'weight' => $this->kind->weight(),
            // RN-09: o contrato não valida o event_id; sem evento no Laravel → null.
            'event' => $this->event ? ['id' => $this->event->id, 'name' => $this->event->name] : null,
            'issuer' => $this->issuer,
            'status' => $this->status,
            'tx_hash' => $this->tx_hash,
            'ledger' => $this->ledger,
            'issued_at' => $this->issued_at?->utc()->format('Y-m-d\TH:i:s\Z'),
            'revoked_at' => $this->revoked_at?->utc()->format('Y-m-d\TH:i:s\Z'),
        ];
    }
}
