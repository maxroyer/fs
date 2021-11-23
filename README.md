# fs

fs is a simple program for sharing files between devices on a local network.
To start a server listening for files on the local network:

    fs rec
An output directory can be specified with the optional tag "-o":

    fs rec -o testdir/output

When a server is currently listening on the local network, a file can be sent to this server with the following:

    fs send ADDRESS FILEPATH
