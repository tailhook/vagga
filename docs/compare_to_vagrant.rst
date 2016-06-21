================
Vagga vs Vagrant
================

Both products do development environments easy to setup. However, there is a big
difference on how they do their work.


Containers
==========

While vagrant emulates full virtual machine, vagga uses linux containers. So
you don't need hardware virtualization and a supervisor. So usually vagga
is more light on resources.

Also comparing to vagrant where you run project inside a virtual machine,
vagga is suited to run commands inside a container, not a full virtual machine
with SSH. In fact many vagga virtual machines don't have a shell and/or a
package manager inside.


Commands
========

While vagrant is concentrated around ``vagrant up`` and VM boot process.  Light
containers allows you to test your project in multiple environments in fraction
of second without waiting for boot or having many huge processes hanging
around.

So instead of having ``vagrant up`` and ``vagrant ssh`` we have user-defined
commands like ``vagga build`` or ``vagga run`` or
``vagga build-a-release-tarball``.


Linux-only
==========

While vagrant works everywhere, vagga only works on linux systems with recent
kernel and userspace utilities.

If you use a mac, just run vagga inside a vagrant container, just like you
used to run docker :)


Half-isolation
==============

Being only a container allows vagga to share memory with host system, which
is usually a good thing.

Memory and CPU usage limits can be enforced on vagga programs using cgroups,
just like on any other process in linux. Vagga runs only on quite recent
linux kernels, which has much more limit capabilities than previous ones.

Also while vagrant allows to forward selected network ports, vagga by default
shares network interface with the host system. Isolating and forwarding
ports will be implemented soon.


.. _vagrant: http://vagrantup.com
