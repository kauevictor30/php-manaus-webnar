<?php

use App\Enums\BadgeKind;
use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('badge_records', function (Blueprint $table) {
            $table->id();
            $table->string('rpc_event_id')->unique();
            $table->string('member_address', 56)->index();
            $table->enum('kind', array_column(BadgeKind::cases(), 'value'));
            $table->unsignedInteger('event_id')->index();
            $table->string('issuer', 56);
            $table->enum('status', ['issued', 'revoked'])->default('issued');
            $table->string('tx_hash', 64);
            $table->unsignedInteger('ledger');
            $table->timestamp('issued_at');
            $table->timestamp('revoked_at')->nullable();
            $table->timestamps();
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('badge_records');
    }
};
