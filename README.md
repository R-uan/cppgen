### CPP Project Generator
Simple CLI program that creates a C/C++ project with the minimal file structure.
```
  <project-name>
    include/
    build/
    src/
      main.c (or main.cpp)
    CMakeLists.txt
    .gitignore

```

### Usage
```bash
cppgen # (for interactive mode)
cppgen -n <project-name> -l <language>
```
