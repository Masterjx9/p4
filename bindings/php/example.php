<?php

declare(strict_types=1);

require_once __DIR__ . '/src/P4Core.php';

use P4\Core\P4Core;

$core = new P4Core();
echo $core->generateIdentityJson() . PHP_EOL;
