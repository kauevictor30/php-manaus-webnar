<?php

namespace App\Http\Controllers;

use App\Http\Requests\ShowMemberRequest;
use App\Http\Resources\MemberProfileResource;
use App\Models\BadgeRecord;
use App\Models\Member;

class MemberController extends Controller
{
    public function show(ShowMemberRequest $request): MemberProfileResource
    {
        $address = $request->validated('address');

        $badges = BadgeRecord::query()
            ->with('event:id,name')
            ->where('member_address', $address)
            ->orderByDesc('ledger')
            ->orderByDesc('id')
            ->get();

        return new MemberProfileResource([
            'address' => $address,
            'member' => Member::where('address', $address)->first(),
            'badges' => $badges,
        ]);
    }
}
