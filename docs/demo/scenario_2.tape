Set Shell "bash"
Set FontSize 20
Set Width 900
Set Height 500
Set LineHeight 1.4
Set Padding 20
Set TypingSpeed 100ms

Sleep 300ms
Type "# We have test-lib dependency inside test-app project and want to modify and test it locally using 'Scenario 2: publish->link'"
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
Type "# test-lib - let's check and publish initial version"
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
Type "# test-app - now we want to use our test-lib"
Sleep 300ms
Enter
Sleep 1s
Type "tree -L 3 -a"
Sleep 300ms
Enter
Sleep 2s

# only for add
Type "kley link test-lib"
Sleep 300ms
Enter
Sleep 2s

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
Type "# test-lib - let's update the library and publish it again"
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
Type "# test-app - try to use updated dependency"
Sleep 300ms
Enter
Sleep 1s

Type "node index.mjs"
Sleep 300ms
Enter
Sleep 2s
Type "# test-app - as you can see the dependency was updated inside our project"
Sleep 300ms
Enter
Sleep 2s
