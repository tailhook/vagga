from sphinxcontrib.domaintools import custom_domain


def setup(app):
    app.add_domain(custom_domain('VaggaConfig',
        name  = 'vagga',
        label = "Vagga Config",

        elements = dict(
            opt = dict(
                objname      = "Yaml Option",
                indextemplate = "pair: %s; Option",
            ),
            cmd = dict(
                objname      = "Vagga Command",
                indextemplate = "pair: %s; Command",
            ),
            volume = dict(
                objname      = "Volume Type",
                indextemplate = "pair: %s; Volume Type",
            ),
            step = dict(
                objname       = "Build Step",
                indextemplate = "pair: %s; Build Step",
            ),
        )))
