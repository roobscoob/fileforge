There are 3 main kinds of arrows.
the most common is Down and Left, where the arrow goes down and to the left.
it would look something like the following:

|----------------------|
|                      |
|          ╰─┬─────╯   |
|    ╭───────╯         |
|    ╰-> Data!!        |
|                      |
|----------------------|

When in this mode (DaL mode), we will ALWAYS place a spacer just
to the left of the branch point (branch point being where the arrow comes from the cradle)

however, sometimes we wish to place the left-most line in a place where going down and left makes no sense,
like the following:

|----------------------|
|                      |
|   ╰─┬─────╯          |
|    ╭╯                |
|    ╰→ Data!!         |
|                      |
|----------------------|

Even though this is following the rules that we set for ourselves in 
DaL mode (that there must always be a spacer), it looks bad!

so if the padding would result in a line *INSIDE OF* our cradle_width (not including the curves on the left and right)
we go into straight mode, which removes the branch line, for the following result:

|----------------------|
|    TEXT!!!           |
|   ╰┬──────╯          |
|    ╰→ Data!!        |
|                      |
|                      |
|----------------------|

However, on rare occasion (even with padding) our down line will end up
beyond the end of the cradle_width, and should go into DaR mode 
like the following:

|----------------------|
|                      |
| ╰─────┬─╯            |
|       ╰─────-> Data!!|
|                      |
|----------------------|

