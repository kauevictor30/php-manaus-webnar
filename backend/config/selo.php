<?php

return [
    'contract_id' => env('SELO_CONTRACT_ID'),
    'rpc_url' => env('SELO_RPC_URL', 'https://soroban-testnet.stellar.org'),
    'start_ledger' => (int) env('SELO_START_LEDGER', 0),
    'admin_token' => env('SELO_ADMIN_TOKEN', ''),
    'frontend_url' => env('FRONTEND_URL', 'http://localhost:5173'),
];
