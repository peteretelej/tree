tree(1) - Linux man page
Name
tree - list contents of directories in a tree-like format.

Synopsis
tree [-adfghilnopqrstuvxACDFNS] [-L level [-R]] [-H baseHREF] [-T title] [-o filename] [--nolinks] [-P pattern] [-I pattern] [--inodes] [--device] [--noreport] [--dirsfirst] [--version] [--help] [--filelimit #] [directory ...]

Description
Tree is a recursive directory listing program that produces a depth indented listing of files. Color is supported ala dircolors if the LS_COLORS environment variable is set, output is to a tty, and the -C flag is used. With no arguments, tree lists the files in the current directory. When directory arguments are given, tree lists all the files and/or directories found in the given directories each in turn. Upon completion of listing all files/directories found, tree returns the total number of files and/or directories listed.

By default, when a symbolic link is encountered, the path that the symbolic link refers to is printed after the name of the link in the format:

name -> real-path

If the '-l' option is given and the symbolic link refers to an actual directory, then tree will follow the path of the symbolic link as if it were a real directory.

Options
Tree understands the following command line switches:

--help
Outputs a verbose usage listing.

--version
Outputs the version of tree.
-a
All files are printed. By default tree does not print hidden files (those beginning with a dot '.'). In no event does tree print the file system constructs '.' (current directory) and '..' (previous directory).

-d

List directories only.

-f

Prints the full path prefix for each file.

-i

Makes tree not print the indentation lines, useful when used in conjunction with the -f option.

-l

Follows symbolic links if they point to directories, as if they were directories. Symbolic links that will result in recursion are avoided when detected.

-x

Stay on the current file-system only. Ala find -xdev.

-P pattern
List only those files that match the wild-card pattern. Note: you must use the -a option to also consider those files beginning with a dot '.' for matching. Valid wildcard operators are '\*' (any zero or more characters), '?' (any single character), '[...]' (any single character listed between brackets (optional - (dash) for character range may be used: ex: [A-Z]), and '[^...]' (any single character not listed in brackets) and '|' separates alternate patterns.
-I pattern
Do not list those files that match the wild-card pattern.
--noreport
Omits printing of the file and directory report at the end of the tree listing.
-p
Print the file type and permissions for each file (as per ls -l).

-s

Print the size of each file in bytes along with the name.

-h

Print the size of each file but in a more human readable way, e.g. appending a size letter for kilobytes (K), megabytes (M), gigabytes (G), terrabytes (T), petabytes (P) and exabytes (E).

-u

Print the username, or UID # if no username is available, of the file.

-g

Print the group name, or GID # if no group name is available, of the file.

-D

Print the date of the last modification time for the file listed.

--inodes
Prints the inode number of the file or directory
--device
Prints the device number to which the file or directory belongs
-F
Append a '/' for directories, a '=' for socket files, a '\*' for executable files and a '|' for FIFO's, as per ls -F

-q

Print non-printable characters in filenames as question marks instead of the default caret notation.

-N

Print non-printable characters as is instead of the default caret notation.

-v

Sort the output by version.

-r

Sort the output in reverse alphabetic order.

-t

Sort the output by last modification time instead of alphabetically.

--dirsfirst
List directories before files.
-n
Turn colorization off always, over-ridden by the -C option.

-C

Turn colorization on always, using built-in color defaults if the LS_COLORS environment variable is not set. Useful to colorize output to a pipe.

-A

Turn on ANSI line graphics hack when printing the indentation lines.

-S

Turn on ASCII line graphics (useful when using linux console mode fonts). This option is now equivalent to '--charset=IBM437' and will eventually be depreciated.

-L level
Max display depth of the directory tree.
--filelimit #
Do not descend directories that contain more than # entries.
-R
Recursively cross down the tree each level directories (see -L option), and at each of them execute tree again adding '-o 00Tree.html' as a new option.

-H baseHREF
Turn on HTML output, including HTTP references. Useful for ftp sites. baseHREF gives the base ftp location when using HTML output. That is, the local directory may be '/local/ftp/pub', but it must be referenced as 'ftp://hostname.organization.domain/pub' (baseHREF should be 'ftp://hostname.organization.domain'). Hint: don't use ANSI lines with this option, and don't give more than one directory in the directory list. If you wish to use colors via CCS stylesheet, use the -C option in addition to this option to force color output.
-T title
Sets the title and H1 header string in HTML output mode.
--charset charset
Set the character set to use when outputting HTML and for line drawing.
--nolinks
Turns off hyperlinks in HTML output.
-o filename
Send output to filename.
Files
/etc/DIR_COLORS
System color database.

~/.dircolors

Users color database.

Environment
LS_COLORS
Color information created by dircolors

TREE_CHARSET

Character set for tree to use in HTML mode.

LC_CTYPE

Locale for filename output.

Bugs
Tree does not prune "empty" directories when the -P and -I options are used. Tree prints directories as it comes to them, so cannot accumulate information on files and directories beneath the directory it is printing.

The -h option rounds to the nearest whole number unlike the ls implementation of -h which rounds up always. The IEC standard names for powers of 2 cooresponding to metric powers of 10 (KiBi, et al.) are silly.

Pruning files and directories with the -I, -P and --filelimit options will lead to incorrect file/directory count reports.
