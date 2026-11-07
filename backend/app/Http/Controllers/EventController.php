<?php

namespace App\Http\Controllers;

use App\Http\Requests\StoreEventRequest;
use App\Http\Resources\EventResource;
use App\Models\Event;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Resources\Json\AnonymousResourceCollection;

class EventController extends Controller
{
    public function index(): AnonymousResourceCollection
    {
        return EventResource::collection(Event::query()->orderBy('date')->orderBy('id')->get());
    }

    public function store(StoreEventRequest $request): JsonResponse
    {
        $event = Event::create($request->validated());

        return (new EventResource($event))->response()->setStatusCode(201);
    }
}
