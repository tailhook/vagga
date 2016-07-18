from django.db import migrations
from django.contrib.auth.hashers import make_password


def create_admin_user(apps, schema_editor):
    User = apps.get_model("auth", "User")
    User.objects.create(username='admin',
                        email='admin@example.com',
                        password=make_password('change_me'),
                        is_superuser=True,
                        is_staff=True,
                        is_active=True)


class Migration(migrations.Migration):

    dependencies = [
        ('auth', '__latest__')
    ]

    operations = [
        migrations.RunPython(create_admin_user)
    ]
