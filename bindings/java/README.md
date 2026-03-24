# Java SDK (JNA)

Maven module with JNA wrapper class:
- [Pp2pCore.java](/c:/Users/RKerrigan/Projects/pp2p/bindings/java/src/main/java/io/github/masterjx9/pp2p/Pp2pCore.java)

Namespace/groupId configured:
- `io.github.masterjx9`

## Build

Build native core from repo root first:
```bash
./scripts/build_pp2p_core_unix.sh
```
or on Windows:
```powershell
.\scripts\build_pp2p_core.ps1
```

Then build Java module:
```bash
cd bindings/java
mvn package
```

## Usage

```java
import io.github.masterjx9.pp2p.Pp2pCore;

Pp2pCore core = new Pp2pCore("C:/path/to/pp2p_core.dll");
String identityJson = core.generateIdentityJson();
```
