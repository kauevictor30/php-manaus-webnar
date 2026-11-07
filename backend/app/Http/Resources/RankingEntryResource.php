<?php

namespace App\Http\Resources;

use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

/** Linha do ranking (design §3.3); recebe uma linha agregada da consulta. */
class RankingEntryResource extends JsonResource
{
    /**
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        return [
            'position' => (int) $this->resource->position,
            'address' => $this->resource->member_address,
            'nickname' => $this->resource->nickname,
            'points' => (int) $this->resource->points,
            'badges' => (int) $this->resource->badges,
        ];
    }
}
