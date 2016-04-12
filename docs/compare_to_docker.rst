===============
Vagga vs Docker
===============

Both products use linux namespaces (a/k/a linux containers) to the work.
However, docker requires root privileges to run, and doesn't allow to make
development environments as easy as vagga.


User Namespaces
===============

As you might noticed that adding user to ``docker`` group (if your docker
socket is accessed by ``docker`` group), is just like giving him a paswordless
``sudo``. This is because root user in docker container is same root that one
on host. Also user that can start docker container can mount arbitrary folder
in host filesystem into the container (So he can just mount ``/etc`` and change
``/etc/passwd``).

Vagga is different as it uses a user namespaces and don't need any programs
running as root or setuid programs or sudo (except systems' builtin
``newuidmap``/``newgidmap`` if you want more that one user inside a container,
but ``newuidmap`` setuid binary is very small functionally and safe).


No Central Daemon
=================

Vagga keeps your containers in ``.vagga`` dir inside your project.
And runs them just like any other command from your shell. I.e. command
run with vagga is child of your shell, and if that process is finished or
killed, its just done. No need to delete container in some central daemon
like docker has (i.e. docker doesn't always remove containers even when
using ``--rm``).

Docker also shares some daemon configuration between different containers
even run by different users. There is no such sharing in vagga.

Also not having central daemon shared between users allows us to have a
user-defined settings file in ``$HOME/.config/vagga/``.


Children Processes
==================

Running processes as children of current shell has following advantages:

* You can monitor process and restart when dead (needs polling in docker),
  in fact there a command type ``supervise`` that does it for you)
* File descriptors may be passed to process
* Processes/containers may be socket-activated (e.g. using ``systemd --user``)
* Stdout and stderr streams are just inherited file descriptors, and they are
  separate (docker mixes the two; it also does expensive copying of the stream
  from the container to the client using HTTP api)


Filesystems
===========

All files in vagga is kept in ``.vagga/container_name/`` so you can inspect all
*persistent* filesystems easily, without finding cryptic names in some system
location, and without sudo


Filesystem Permissions
======================

Docker by default runs programs in container as root. And it's also a root on
the host system. So usually in your development project you get files with root
owner. While it's possible to specify your uid as a user for running a
process in container, it's not possible to do it portable. I.e. your uid
in docker container should have ``passwd`` entry. And somebody else may
have another uid so must have a different entry in ``/etc/passwd``. Also if
some process realy needs to be root inside the container (e.g. it must spawn
processes by different users) you just can't fix it.

.. note:: In fact you can specify `uid` without adding a ``passwd`` entry, and
   that works most of the time. Up to the point some utility needs to
   lookup info about user.

With help of user namespaces Vagga runs programs as a root inside a container,
but it looks like your user outside. So all your files in project dir are still
owned by you.


Security
========

While docker has enterprise support, including security updates. Vagga doesn't
have such (yet).

However, Vagga runs nothing with root privileges. So even running root process
in guest system is at least as secure as running any unprivileged program in
host sytem. It also uses chroot and linux namespaces for more isolation.
Compare it to docker which doesn't consider running as root inside a container
secure.

You can apply selinux or apparmor rules for both.


Filesystem Redundancy
=====================

Vagga creates each container in ``.vagga`` as a separate directory. So
theoretically it uses more space than layered containers in docker. But if you
put that dir on ``btrfs`` filesystem you can use bedup_ to achieve much
better redundancy than what docker provides.


.. _docker: http://docker.io
.. _bedup:  https://github.com/g2p/bedup
