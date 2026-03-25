<?php

declare(strict_types=1);

require_once __DIR__ . '/src/Pp2pCore.php';

use Pp2p\Core\Pp2pCore;

$core = new Pp2pCore();
echo $core->generateIdentityJson() . PHP_EOL;
