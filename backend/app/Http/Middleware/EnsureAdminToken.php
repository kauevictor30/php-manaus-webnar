<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

/**
 * Protege rotas administrativas com o header X-Admin-Token (design §3.3, §5).
 * Comparação em tempo constante com hash_equals; 401 sem detalhes.
 */
class EnsureAdminToken
{
    public function handle(Request $request, Closure $next): Response
    {
        $expected = (string) config('selo.admin_token');
        $given = $request->header('X-Admin-Token');

        if ($expected === '' || ! is_string($given) || ! hash_equals($expected, $given)) {
            return response()->json(['message' => 'Unauthorized'], 401);
        }

        return $next($request);
    }
}
