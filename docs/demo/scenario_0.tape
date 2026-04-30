Set Shell "bash"
Set FontSize 20
Set Width 900
Set Height 500
Set LineHeight 1.4
Set Padding 20
Set TypingSpeed 100ms

Sleep 300ms
Type "# Basic workflow: `publish` in the library, then `install` in the project."
Sleep 300ms
Enter
Sleep 2s

Type "cd test-lib"
Sleep 300ms
Enter
Type "tree -L 3 -a"
Sleep 300ms
Enter
Sleep 2s
Type "# 1. test-lib: Check and publish the initial version."
Sleep 300ms
Enter
Sleep 1s
Type "cat index.mjs"
Sleep 300ms
Enter
Sleep 2s
Type "kley publish"
Sleep 300ms
Enter
Sleep 2s
Type "cd ../test-app"
Sleep 300ms
Enter
Type "# 2. test-app: Use the library with a single command."
Sleep 300ms
Enter
Sleep 1s
Type "tree -L 3 -a"
Sleep 300ms
Enter
Sleep 2s

Type "kley install test-lib"
Sleep 300ms
Enter
Sleep 3s

Type "cat index.mjs"
Sleep 300ms
Enter
Sleep 2s
Type "node index.mjs"
Sleep 300ms
Enter
Sleep 3s

Type "cd ../test-lib"
Sleep 300ms
Enter
Sleep 1s
Type "# 3. test-lib: Update the library and push the changes."
Sleep 300ms
Enter
Sleep 1s
Type "vim index.mjs"
Sleep 300ms
Enter
Sleep 1s
Type "j"
Sleep 500ms
Type "yyp"
Sleep 500ms
Type "wwww"
Sleep 300ms
Type "inew line from "
Sleep 300ms
Escape
Sleep 1s
Type ":"
Type "w"
Type "q"
Sleep 500ms
Enter
Sleep 2s
Type "kley publish --push"
Sleep 300ms
Enter
Sleep 2s
Type "cd ../test-app"
Sleep 300ms
Enter
Sleep 1s
Type "# 4. test-app: The dependency is updated automatically."
Sleep 300ms
Enter
Sleep 1s

Type "node index.mjs"
Sleep 300ms
Enter
Sleep 2s
Type "# As you can see, the dependency was updated instantly."
Sleep 300ms
Enter
Sleep 2s
