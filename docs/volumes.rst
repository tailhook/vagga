.. _volumes:

=======
Volumes
=======

Volumes define some additional filesystems to mount inside container. The
default configuration is similar to the following:

.. code-block:: yaml

    volumes:
        /tmp: !Tmpfs
            size: 100Mi
            mode: 0o1777

Available volume types:

``!Tmpfs``
    Mounts ``tmpfs`` filesystem. There are two parameters for this kind of
    volume: ``size`` -- limit for filesystem size in bytes. You may use
    suffixes ``k, M, G, ki, Mi, Gi`` for bigger units. The ones with ``i``
    are for power of two units, the other ones are for power of ten.

``!VaggaBin``
    Mounts vagga binary directory inside the container (usually it's contained
    in ``/usr/lib/vagga`` in host system). This may be needed for
    network_testing_ or may be for vagga in vagga (i.e. container in container)
    use cases.


