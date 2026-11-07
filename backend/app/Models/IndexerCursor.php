<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class IndexerCursor extends Model
{
    protected $fillable = ['contract_id', 'cursor'];
}
