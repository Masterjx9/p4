<?php

declare(strict_types=1);

require_once __DIR__ . '/src/Pp2pCore.php';

use Pp2p\Core\Pp2pCore;

$lib = getenv('PP2P_CORE_LIB');
if ($lib === false || $lib === '') {
    if (PHP_OS_FAMILY === "Windows") {
        $lib = __DIR__ . "/../../dist/pp2p_core/windows-x64/pp2p_core.dll";
    } elseif (PHP_OS_FAMILY === "Darwin") {
        $lib = __DIR__ . "/../../dist/pp2p_core/macos/libpp2p_core.dylib";
    } else {
        $lib = __DIR__ . "/../../dist/pp2p_core/linux-x64/libpp2p_core.so";
    }
}

$core = new Pp2pCore($lib);
echo $core->generateIdentityJson() . PHP_EOL;
