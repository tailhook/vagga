"""init

Revision ID: 360aff35f0d
Revises: 
Create Date: 2015-10-30 15:24:23.181676

"""

# revision identifiers, used by Alembic.
revision = '360aff35f0d'
down_revision = None
branch_labels = None
depends_on = None

from alembic import op
import sqlalchemy as sa


def upgrade():
    op.create_table('tbl',
    sa.Column('id', sa.Integer(), nullable=False),
    sa.Column('name', sa.String(length=256), nullable=False),
    sa.PrimaryKeyConstraint('id')
    )


def downgrade():
    op.drop_table('tbl')
