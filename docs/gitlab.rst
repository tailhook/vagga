Using in Gitlab CI
==================

Vagga's defaults are very conservative and are better suited for workstations.
For higher performance on CI you need some settings activated.

For example, here is good settings for Gitlab CI:

.. code-block:: yaml

   storage-dir: /var/lib/gitlab-runner/containers
   storage-subdir-from-env-var: CI_PROJECT_NAME
   cache-dir: /var/lib/gitlab-runner/cache
   ubuntu-mirror: http://ua.archive.ubuntu.com/
   alpine-mirror: http://dl-cdn.alpinelinux.org/alpine/
   build-lock-wait: true
   index-all-images: true
   hard-link-identical-files: true
   ubuntu-skip-locking: true
   hard-link-between-projects: true
   versioned-build-dir: true

Note: some of these settings are non-secure if single gitlab runner is used
across organizations. See :ref:`Settings` for more info on each setting.

Gitlab-runner config:

.. code-block:: toml

   concurrent = 3
   check_interval = 0

   [[runners]]
     name = "vagga runner"
     url = "https://lab.thinkglobal.space"
     token = "xxx_yourToken_xxx"
     executor = "shell"
     pre_build_script = "vagga --version"
     [runners.cache]

Note: concurrency > 1 is okay, pre_build_script shows vagga version

Also it's a good idea to make cleanup regularly::

   vagga _clean --global --unused --al-least 7days

(run as ``gitlab-runner`` user)
