.. default-domain:: vagga

.. _settings:

========
Settings
========


Global Settings
===============


Settings are searched for in one of the following files:

 * ``$HOME/.config/vagga/settings.yaml``
 * ``$HOME/.vagga/settings.yaml``
 * ``$HOME/.vagga.yaml``


Supported settings:


.. opt:: storage-dir

    Directory where to put images build by vagga. Usually they are stored in
    ``.vagga`` subdirectory of the project dir. It's mostly useful when the
    ``storage-dir`` points to a directory on a separate partition. Path may
    start with ``~/`` which means path is inside the user's home directory.

.. opt:: storage-subdir-from-env-var

    This options is designed specifically for Continuous Integration (CI)
    systems. When this option is set it identifies an environment variable
    that is used to specify the name of a subdirectory of the storage dir used
    for current project. It's only useful if :opt:`storage-dir` is set.

    For example, for gitlab you may want to set::

        storage-subdir-from-env-var: CI_PROJECT_NAME

    Or or alternatively::

        storage-subdir-from-env-var: CI_PROJECT_NAMESPACE

    Note: only dash, underscore, and alphanumerc chars are allowed in the name,
    all other characters will be replaced by dash (so technically clashes of
    names are possible). If this environment variable is empty, then vagga
    will fail.

    Note 2: already set symlink in `.vagga/.lnk` overrides this setting. This
    also means that `_init_storage_dir` overiddes the setting. Which means you
    may not get immediate result when migrating from old system. You may
    remove the link though, if your CI system does not do that by default.

    .. versionadded:: 0.7.2

.. opt:: cache-dir

    Directory where to put cache files during the build. This is used to speed
    up the build process. By default cache is put into ``.vagga/.cache`` in
    project directory but this setting allows to have cache directory shared
    between multiple projects. Path may start with ``~/`` which means path is
    inside the user's home directory.

.. opt:: site-settings

    The mapping of project paths to settings for this specific
    project.

    Example:

    .. code-block:: yaml

       site-settings:
         /home/myuser/myproject:
           cache-dir: /home/myuser/.cache/myproject

.. opt:: proxy-env-vars

    Enable forwarding for proxy environment variables. Default ``true``.
    Environment variables currently that this setting influence currently:
    ``http_proxy``, ``https_proxy``, ``ftp_proxy``, ``all_proxy``,
    ``no_proxy``.

.. opt:: propagate-environ

    A list of variables and patterns that are propagated into the container
    by default. Example:

    .. code-block:: yaml

        propagate-environ:
        - "GIT_BRANCH"
        - "JENKINS_*"
        - "CI_*"

    This is intended to use on CI system where parameters of build job is
    safe to propagate.

    While technically you can specify `"*"` it's very dangerous and
    error-prone option to enable.

.. opt:: external-volumes

   A mapping of volume names to the directories inside the host file system.

   .. note:: The directories must exist even if unused in any ``vagga.yaml``.

   For example, here is how you might export home:

   .. code-block:: yaml

      external-volumes:
        home: /home/user

   Then in `vagga.yaml` you use it as follows (prepend with `/volumes`):

   .. code-block:: yaml

      volumes:
        /root: !BindRW /volumes/home

   See :ref:`volumes` for more info about defining mount points.

   .. warning::

      1. Usage of volume is usually a subject for filesystem permissions. I.e.
         your user becomes `root` inside the container, and many system users
         are not mapped (not present) in container at all. This means that
         mounting `/var/lib/mysql` or something like that is useless, unless
         you chown the directory

      2. Any vagga project may use the volume if it's defined in global
         config. You may specify the volume in :opt:`site-settings` if you
         care about security (and you should).

.. opt:: push-image-script

   A script to use for uploading a container image when you run
   `vagga _push_image`.

   To push image using webdav::

       push-image-script: "curl -T ${image_path} \
           http://example.org/${container_name}.${short_hash}.tar.xz"

   To push image using `scp` utility (SFTP protocol)::

       push-image-script: "scp ${image_path} \
          user@example.org:/target/path/${container_name}.${short_hash}.tar.xz"

   The FTP(s) (for example, using `lftp` utility) or S3 (using `s3cmd`) are
   also valid choices.

   .. note:: This is that rare case where command is run by vagga in your host
      filesystem. This allows you to use your credentials in home directory,
      and ssh-agent's socket. But also this means that utility to upload
      images must be installed in host system.

   Variables:

   container_name
       The name of the container as declared in `vagga.yaml`

   short_hash
       The short hash of container setup. This is the same hash that is used
       to detect whether container configuration changed and is needed to
       be rebuilt. And the same hash used in directory name `.vagga/.roots`.

.. opt:: auto-apply-sysctl

    Set sysctls required by command. We do our best to only apply "safe"
    sysctls by vagga automatically. Still it may exhaust resources of your
    system, so use this option on your own risk.

    We apply settings with ``sudo -k`` which means it will prompt for password
    each time setting is tuned (probably only after system reboot).

    Settings currently exists:

    ============================= ============================= ===============
    Key in vagga.yaml             Sysctl Name                   Hardcoded Limit
    ============================= ============================= ===============
    :opt:`expect-inotify-limit`   fs.inotify.max_user_watches   524288
    ============================= ============================= ===============

All project-local settings are also allowed here.


Project-Local Settings
======================

Project-local settings may be in the project dir in:

 * ``.vagga.settings.yaml``
 * ``.vagga/settings.yaml``

All project-local settings are also allowed in global config.

While settings can potentially be checked-in to version control it's advised
not to do so.

.. opt:: version-check

    If set to ``true`` (default) vagga will check if the container that is
    already built is up to date with config. If set to ``false`` vagga will
    use any container with same name already built. It's only useful for
    scripts for performance reasons or if you don't have internet and
    containers are not too outdated.

.. opt:: ubuntu-mirror

    Set to your preferred ubuntu mirror. Default is currently a special
    url ``mirror://mirrors.ubuntu.com/mirrors.txt`` which choses local mirror
    for you. But it sometimes fails. Therefore we reserve an option to change
    the default later.

    The best value for this settings is probably
    ``http://<COUNTRY_CODE>.archive.ubuntu.com/ubuntu/``.

.. opt:: ubuntu-skip-locking

   Enables ``-o Debug::NoLocking=yes``. This is *super-experimental*, but
   allows to build multiple ubuntu images in parallel even when they use the
   same cache (i.e. in the same project, or when using :opt:`cache-dir`)

.. opt:: alpine-mirror

    Set to your preferred alpine mirror. By default it's the random one is
    picked from the list.

    .. note:: Alpine package manager is used not only for building
       :step:`Alpine` distribution, but also internally for fetching tools that
       are outside of the container filesystem (for example to fetch ``git``
       for :step:`Git` or :step:`GitInstall` command(s))

.. opt:: build-lock-wait

    By default (``build-lock-wait: false``) vagga stops current command and
    prints a message when some other process have already started to build the
    image. When this flag is set to ``true`` vagga will wait instead. This
    is mostly useful for CI systems.

.. opt:: environ

    The mapping, that overrides environment variables set in container or command.

.. opt:: run-symlinks-as-commands

    (default ``true``) If the setting is true, when there is a symlink named
    ``yyy`` that points to a vagga, and vagga is run by calling the name of
    that symlink vagga finds a command with ``symlink-name`` which equals to
    this command and runs it directly, passing all the arguments to that
    command (i.e. vagga doesn't try to parse command-line itself).

    .. versionadded:: 0.7.1

.. opt:: index-all-images

    (default ``false``) When the setting is ``true`` then vagga will hash all
    the files inside the containers and will create a special signature file.

    .. versionadded:: 0.7.1

.. opt:: hard-link-identical-files

    **This functionality is experimental**. Use at your own risk.

    (default ``false``) This setting is paired with ``index-all-images``.
    When both settings are ``true`` vagga will search identical files inside
    other containers and will replace the same files with hard links.
    This can significantly reduce a disk space occupied by the containers.

    There are two precautions about this setting:

    1. Date modified, date created and most other metadata is ignored
    2. If you edit file directly in ``.vagga/<container-name>`` you may
       edit files in multiple containers at the same time (this is similar
       to ``transient-hard-link-copy`` in a some way)


    .. versionadded:: 0.7.2

.. opt:: disable-auto-clean

   Disables ``auto-clean`` option in all containers. This is useful on CI
   systems where multiple parallel builds should work.

.. opt:: versioned-build-dir

   (default ``false``) When building container, say ``mycontainer`` by default
   we use ``.tmp.mycontainer`` dir for building. This settings enables naming
   dir ``.tmp.mycontainer.1a2b3c4`` where ``1a2b3c4`` is a container version
   being built.

   Note: for some containers where we can't determine version before building
   a container this setting does nothing.

   It's useful to turn this setting on on CI systems with configured
   ``storage_dir``, when multiple versions of a single container could be
   being built simultaneously. It makes little sense to enable it on
   a workstation.



