=====
Vagga
=====


Vagga is a fully-userspace container system inspired by Vagrant_ and Docker_,
mainly used for development system.

Mayor Features Are:

* Running programs in linux containers (not a full virtualization like Vagrant)
* Fully userspace containers, no need for elevated privileges like Docker_
* Runs containerized process as a child of current shell, no attach/detach hell
* Images are automatically rebuilt and versioned
* Vagga has tools to manage trees of containers and trees of processes inside
* Can build a docker container from vagga environment

Status: Work in progress


Vagga vs Docker
===============


User Namespaces
---------------

As you might noticed that adding user to ``docker`` group, is just like giving
him a paswordless ``sudo``. This is because root user in docker container is
same root that one on host. Also user that can start docker container can
mount arbitrary folder in host filesystem into the container (So he can
just mount ``/etc`` and change ``/etc/passwd``).

Vagga is different as it uses a user namespaces and don't need any programs
running as root or setuid programs or sudo (except systems' builtin
``newuidmap``/``newgidmap`` if you want more that one user inside a container,
but ``newuidmap`` setuid binary is very small functionally and safe).


No Central Daemon
-----------------

Vagga keeps your containers in ``.vagga`` dir inside your project.
And runs them just like any other command from your shell. I.e. command
run with vagga is child of your shell, and if that process is finished or
killed, its just done. No need to delete container in some central daemon
like docker has (i.e. docker doesn't always remove containers even when
using ``--rm``).

Docker also shares some daemon configuration between different containers
even run by different users. There is no such sharing in vagga.


Children Processes
------------------

Running processes as children of current shell has following advantages:

* You can monitor process and restart when dead (needs polling in docker)
* File descriptors may be passed to process
* Processes/containers may be socket-activated (e.g. using ``systemd --user``)
* Stdout and stderr streams are just inherited file descriptors, and they are
  separate (docker mixes the two, and also copies real stream to client one)


Filesystems
-----------

All files in vagga is kept in ``.vagga`` so you can inspect all *persistent*
filesystems easily, without finding cryptic names in some system location,
and without sudo


Filesystem Permissions
----------------------

Docker by default runs programs in container as root. And it's also a root on
the host system. So usually in your development project you get files with root
owner. While it's possible to specify your uid as a user for running a
process in container, it's not possible to have it portable. I.e. your uid
in docker container should have ``passwd`` entry. And somebody else may
have another uid so must have a different entry in ``/etc/passwd``.


With help of user namespaces Vagga runs programs as a root inside a container,
but it looks like your user outside. So all your files in project dir are still
owned by you.


Security
--------

While docker has enterprise support, including security updates. Vagga doesn't
have such (yet).

However, Vagga runs nothing with root privileges. So even running root process
in guest system it's at least as secure as running any unprivileged program in
host sytem. It also uses chroot and linux namespaces for more isolation.
Compare it to docker which doesn't consider running as root inside a container
secure.


Filesystem Redundancy
---------------------

Vagga creates each container in ``.vagga`` as a separate directory. So
theoretically it uses more space than layered containers in docker. But if you
put that dir on ``btrfs`` filesystem you can use bedup_ to achieve much
better redundancy than what docker provides.



Vagga vs Vagrant
================

Both products do development enviroments easy to setup. However, there is a big
difference on how they do their work.


Containers
----------

While vagrant emulates full virtual machine, vagga uses linux containers. So
you don't need hardware virtualization and a supervisor. So usually vagga
is more light on resources.

Also comparing to vagrant where you run project inside a virtual machine,
vagga is suited to run commands inside a container, not a full virtual machine
with SSH. In fact many vagga virtual machines don't have a shell and/or a
package manager inside.


Commands
--------

While vagrant is concentrated around ``vagrant up`` and VM boot process.
Light containers allows you to test your project in multiple environments
in seconds without waiting for boot or having many huge processes hanging
around.

So instead of having ``vagrant up`` and ``vagrant ssh`` we have user-defined
commands like ``vagga build`` or ``vagga run`` or
``vagga build-a-release-tarball``.


Linux-only
----------

While vagrant works everywhere, vagga only works on linux systems with recent
kernel and userspace utulities.

If you use a mac, just run vagga inside a vagrant container, just like you
used to run docker :)


Half-isolation
--------------

Being only a container allows vagga to share memory with host system, which
is usually a good thing.

Memory and CPU usage limits can be enforced on vagga programs using cgroups,
just like on any other process in linux. Vagga runs only on quite recent
linux kernels, which has much more limit capabilities than previous ones.

Also while vagrant allows to forward selected network ports, vagga by default
shares network interface with the host system. Isolating anf forwarding
ports will be implemented soon.


.. _vagrant: http://vagrantup.com
.. _docker: http://docker.io
.. _bedup:  https://github.com/g2p/bedup
