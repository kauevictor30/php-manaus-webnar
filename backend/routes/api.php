<?php

use App\Http\Controllers\EventController;
use App\Http\Middleware\EnsureAdminToken;
use Illuminate\Support\Facades\Route;

Route::get('/events', [EventController::class, 'index']);
Route::post('/events', [EventController::class, 'store'])->middleware(EnsureAdminToken::class);
