<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('indexer_cursors', function (Blueprint $table) {
            $table->id();
            $table->string('contract_id', 56)->unique();
            $table->string('cursor');
            $table->timestamps();
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('indexer_cursors');
    }
};
