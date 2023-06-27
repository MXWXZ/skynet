export default {
  'pages.404.text': 'Sorry, the page you visited does not exist.',
  'pages.404.backhome': 'Back Home',

  'pages.403.text': "Sorry, you don't have access to this page.",
  'pages.403.backhome': 'Back Home',

  'pages.index.subtitle': 'Big Brother is Watching You',
  'pages.index.username': 'Username',
  'pages.index.username.required': 'Please input username',
  'pages.index.password': 'Password',
  'pages.index.password.required': 'Please input password',
  'pages.index.rememberme': 'Remember me',
  'pages.index.captcha.required': 'Please verify you are not bot',

  'pages.system.content': 'Manage Skynet system',
  'pages.system.dangerzone.title': 'Danger Zone',
  'pages.system.dangerzone.shutdown.text': 'Shutdown Skynet',
  'pages.system.dangerzone.shutdown.button': 'Shutdown',
  'pages.system.dangerzone.shutdown.title':
    'You need to restart manually if not in daemon mode, confirm?',
  'pages.system.dangerzone.shutdown.content':
    'Program will exit gracefully. You should know what you are doing.',

  'pages.notification.content': 'Manage Skynet notifications',
  'pages.notification.table.level': 'Level',
  'pages.notification.table.name': 'Name',
  'pages.notification.table.message': 'Message',
  'pages.notification.op.deleteall.title': 'Delete all notifications, confirm?',

  'pages.permission.table.name': 'Name',
  'pages.permission.table.note': 'Note',
  'pages.permission.table.perm': 'Permission',
  'pages.permission.table.ban': 'Ban',
  'pages.permission.table.inherit': 'Inherit',
  'pages.permission.op.update': 'Update',
  'pages.permission.table.origin': 'Origin',
  'pages.permission.table.self': 'self',

  'pages.group.content': 'Manage Skynet user group',
  'pages.group.table.note': 'Note',
  'pages.group.table.name': 'Name',
  'pages.group.op.add.title': 'Add New Group',
  'pages.group.op.add.content': 'New group has no permission by default.',
  'pages.group.op.add.user.title': 'Add User to Group',
  'pages.group.op.delete.title': 'Delete group {name}, confirm?',
  'pages.group.op.delete.user.title': "Delete group's user {name}, confirm?",
  'pages.group.op.delete.selected.title': 'Delete selected groups, confirm?',
  'pages.group.op.update.title': 'Update Group',
  'pages.group.op.clone.title': 'Clone Group',
  'pages.group.op.clone.content':
    'Cloned group will have the same permission as the base group.',
  'pages.group.op.update.tip': 'Update',
  'pages.group.op.clone.tip': 'Clone',
  'pages.group.op.perm.tip': 'Permission',
  'pages.group.op.user.tip': 'Manage Users',
  'pages.group.op.perm.title': 'Manage Group Permission',
  'pages.group.op.user.title': 'Manage Group Users',
  'pages.group.form.name.tip': 'Group name is unique',
  'pages.group.form.cloneuser': 'Clone all user',
  'pages.group.form.basegroup': 'Base group',

  'pages.user.content': 'Manage Skynet user',
  'pages.user.table.username': 'Username',
  'pages.user.table.password': 'Password',
  'pages.user.table.lastlogin': 'Last Login',
  'pages.user.table.avatar': 'Avatar',
  'pages.user.table.lastip': 'Last IP',
  'pages.user.form.group': 'Group',
  'pages.user.form.group.tip':
    'Add user to group, this will grant all group permission to user',
  'pages.user.form.username.tip': 'Username is unique',
  'pages.user.form.avatar.upload': 'Upload',
  'pages.user.form.avatar.invalid': '{file} is not allowed',
  'pages.user.form.baseuser': 'Base user',
  'pages.user.form.clonegroup': 'Clone all group',
  'pages.user.form.password.placeholder': 'Left empty for unchange',
  'pages.user.op.clone.tip': 'Clone',
  'pages.user.op.update.tip': 'Update',
  'pages.user.op.kick.tip': 'Kick',
  'pages.user.op.kick.title': 'Kick user {username}, confirm?',
  'pages.user.op.add.title': 'Add New User',
  'pages.user.op.perm.tip': 'Permission',
  'pages.user.op.perm.title': 'Manage User Permission',
  'pages.user.op.add.content': 'New user has no permission by default.',
  'pages.user.op.delete.title': 'Delete user {username}, confirm?',
  'pages.user.op.delete.selected.title': 'Delete selected users, confirm?',
  'pages.user.op.clone.title': 'Clone User',
  'pages.user.op.clone.content':
    'Cloned user will have the same permission as the base user.',
  'pages.user.op.update.title': 'Update User',

  'pages.plugin.content': 'Manage Skynet plugin',
  'pages.plugin.table.name': 'Name',
  'pages.plugin.table.status': 'Status',
  'pages.plugin.table.version': 'Version',
  'pages.plugin.table.path': 'Path',
  'pages.plugin.table.delete.title': 'Delete plugin {name}, confirm?',
  'pages.plugin.table.disabletip': 'Disable',
  'pages.plugin.table.enabletip': 'Enable',
  'pages.plugin.table.enable.title': 'Enable plugin {name}, confirm?',
  'pages.plugin.table.enable.content':
    'You cannot unload the plugin unless restarting Skynet.',
  'pages.plugin.table.disable.title': 'Disable plugin {name}, confirm?',
  'pages.plugin.table.disable.content': 'Restart Skynet to take effect.',
};
