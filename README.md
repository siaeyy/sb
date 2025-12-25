<h1 align="center">
    <img src="logo.png" width="200" height="169"/><br>
</h1>

<h4 align="center">
    A command-line tool to search binaries in Unix-like systems
</h4>

### Motivation
I forget things easily and a lot, the names of command-line utilities/tools are included. When I need one of them and only remember little part of its name, I have to google it or ask a LLM which is I don't like. After few times like this, I knew I need a solution and I started creating this tool. So this is a need for me, I hope it works for you either.

### How does this work?
Simple, it just searches executables (or the way I call them: binaries) in directories which are part of PATH environment variable or a seperated paths argument given by you. Then it compares search input with the names of the binaries by similarity and displays them after ordering. For extra, it can extract descriptions from man pages for found binaries, with "**[man-db](https://man-db.gitlab.io/man-db/)**" and "**[groff](https://www.gnu.org/software/groff/groff.html)**" for displaying them too.

### Demo
Just showing how does it look:

https://github.com/user-attachments/assets/de2becf1-41fb-4386-a2dd-f2e2f104f632

#### The things are missing / should be fixed:
1. Error handling (there is no error handling, really. I just ignored them 😊)
2. (Interactıve mode) Description truncation, a big missing I think
3. (Interactıve mode) Navigation on list, another big missing
4. Codebase explanation. There are no any kind of comments between these lines, sorry
4. Codebase formatting. I was enjoying with adjusting the indentation myself
5. Codebase refactoring. The modularity and quality might be messed up a little bit

~~For a project that aiming personal use, these can be tolerated but not this two:~~

6. Most importantly, better similarity checking.<br>
Another idea is word searching in description,<br>
it would be nice but could be painful to implement
7. And there is a problem with binaries with same similarity (points):<br>
Because of they have same similarity, only one of them can be saved.<br>
I did not think this would be a problem when handling similarity checking<br>
but looks like short search inputs produce this problem oftenly
