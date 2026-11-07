<?php

namespace App\Http\Resources;

use App\Models\BadgeRecord;
use App\Models\Member;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;
use Illuminate\Support\Collection;

/**
 * Perfil off-chain (design §3.3). O resource recebe
 * ['address' => string, 'member' => ?Member, 'badges' => Collection<BadgeRecord>].
 */
class MemberProfileResource extends JsonResource
{
    /**
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        /** @var Member|null $member */
        $member = $this->resource['member'];
        /** @var Collection<int, BadgeRecord> $badges */
        $badges = $this->resource['badges'];

        return [
            'address' => $this->resource['address'],
            'nickname' => $member?->nickname,
            'avatar_url' => $member?->avatar_url,
            // Só badges ativos contam pontos (pesos D-06).
            'points' => $badges
                ->where('status', BadgeRecord::STATUS_ISSUED)
                ->sum(fn (BadgeRecord $b) => $b->kind->weight()),
            'badges' => BadgeRecordResource::collection($badges),
        ];
    }
}
