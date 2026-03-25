# Java SDK (JNA)

Maven module with JNA wrapper class:
- [Pp2pCore.java](/c:/Users/RKerrigan/Projects/pp2p/bindings/java/src/main/java/io/github/masterjx9/pp2p/Pp2pCore.java)

Namespace/groupId configured:
- `io.github.masterjx9`

## Install (Maven)

```xml
<dependency>
  <groupId>io.github.masterjx9</groupId>
  <artifactId>pp2p-core-sdk</artifactId>
  <version>0.2.0</version>
</dependency>
```

## Runtime requirements

- Java 11+
- Bundled native binary is auto-loaded for:
  - Windows x64
  - Linux x64
  - macOS Intel (x64)
  - macOS Apple Silicon (arm64)

## Usage

```java
import io.github.masterjx9.pp2p.Pp2pCore;

Pp2pCore core = new Pp2pCore(); // auto-load bundled native lib
String identityJson = core.generateIdentityJson();
```

Optional override:
- set `PP2P_CORE_LIB` to an absolute path to your own native library.
