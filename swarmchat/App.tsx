import React, { useEffect } from 'react';
import { StatusBar } from 'react-native';
import { Provider, useDispatch } from 'react-redux';
import { store, AppDispatch } from './src/store';
import { loadIdentity } from './src/store/identitySlice';
import AppNavigator from './src/navigation/AppNavigator';

function AppInit() {
  const dispatch = useDispatch<AppDispatch>();

  useEffect(() => {
    dispatch(loadIdentity());
  }, [dispatch]);

  return (
    <>
      <StatusBar barStyle="light-content" backgroundColor="#0D1117" />
      <AppNavigator />
    </>
  );
}

export default function App() {
  return (
    <Provider store={store}>
      <AppInit />
    </Provider>
  );
}
