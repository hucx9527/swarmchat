module.exports = {
  project: {
    ios: {
      sourceDir: './ios',
    },
    android: {
      sourceDir: './android',
      packageName: 'com.swarmchat',
    },
  },
  dependencies: {
    'react-native-camera': {
      platforms: {
        android: {
          sourceDir: './node_modules/react-native-camera/android',
        },
      },
    },
    'react-native-fs': {
      platforms: {
        android: null,
      },
    },
    'react-native-permissions': {
      platforms: {
        android: {
          sourceDir: './node_modules/react-native-permissions/android',
        },
      },
    },
    'react-native-svg': {
      platforms: {
        android: {
          sourceDir: './node_modules/react-native-svg/android',
        },
      },
    },
  },
};
