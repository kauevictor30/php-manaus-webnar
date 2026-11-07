<?php

namespace App\Models;

use App\Enums\BadgeKind;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class BadgeRecord extends Model
{
    public const STATUS_ISSUED = 'issued';

    public const STATUS_REVOKED = 'revoked';

    protected $fillable = [
        'rpc_event_id', 'member_address', 'kind', 'event_id', 'issuer',
        'status', 'tx_hash', 'ledger', 'issued_at', 'revoked_at',
    ];

    protected function casts(): array
    {
        return [
            'kind' => BadgeKind::class,
            'event_id' => 'integer',
            'ledger' => 'integer',
            'issued_at' => 'datetime',
            'revoked_at' => 'datetime',
        ];
    }

    public function event(): BelongsTo
    {
        return $this->belongsTo(Event::class, 'event_id');
    }
}
