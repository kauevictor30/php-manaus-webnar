<?php

use App\Http\Controllers\EventController;
use App\Http\Controllers\MemberController;
use App\Http\Controllers\RankingController;
use App\Http\Middleware\EnsureAdminToken;
use Illuminate\Support\Facades\Route;

Route::get('/events', [EventController::class, 'index']);
Route::post('/events', [EventController::class, 'store'])->middleware(EnsureAdminToken::class);

Route::get('/members/{address}', [MemberController::class, 'show']);
Route::get('/ranking', [RankingController::class, 'index']);
