<?php

namespace App\Http\Controllers;

use App\Enums\BadgeKind;
use App\Http\Resources\RankingEntryResource;
use App\Models\BadgeRecord;
use Illuminate\Http\Resources\Json\AnonymousResourceCollection;
use Illuminate\Support\Facades\DB;

class RankingController extends Controller
{
    private const TOP = 20;

    public function index(): AnonymousResourceCollection
    {
        // SUM(CASE kind WHEN ? THEN ? ...) com os pesos do enum (D-06).
        $cases = [];
        $bindings = [];
        foreach (BadgeKind::cases() as $kind) {
            $cases[] = 'WHEN ? THEN ?';
            array_push($bindings, $kind->value, $kind->weight());
        }
        $pointsSql = 'SUM(CASE badge_records.kind '.implode(' ', $cases).' ELSE 0 END)';

        $rows = DB::table('badge_records')
            ->leftJoin('members', 'members.address', '=', 'badge_records.member_address')
            ->where('badge_records.status', BadgeRecord::STATUS_ISSUED)
            ->groupBy('badge_records.member_address', 'members.nickname')
            ->select('badge_records.member_address', 'members.nickname')
            ->selectRaw("{$pointsSql} as points", $bindings)
            ->selectRaw('COUNT(*) as badges')
            ->orderByDesc('points')
            ->orderByDesc('badges')
            ->orderBy('badge_records.member_address')
            ->limit(self::TOP)
            ->get()
            ->values()
            ->map(function (object $row, int $i) {
                $row->position = $i + 1;

                return $row;
            });

        return RankingEntryResource::collection($rows);
    }
}
