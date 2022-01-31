# battlemap-gen

Tabletop RPG battlemap generator.

Early days yet. My intention is that the program will have these features:

- Several map themes (wilderness, dungeon, etc.). 
- Should serve maps both from a command-line interface and from a web/CGI interface.

One limitation of the current code is that it relies too heavily on loops where I retry until some randomly-created element (like a road or a wall) becomes valid. It's a wasteful brute-force method. I prevent infinite loops with a finite retry count, but this means I might use all my retries without having a valid outcome. I need to alter the algorithm so it's smart enough to randomly generate elements that *must* be valid, eliminating the need for retries.




