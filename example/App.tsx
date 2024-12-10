import { StatusBar } from 'expo-status-bar';
import { StyleSheet, Text, View } from 'react-native';
import 'react-native-automerge';
import * as Automerge from '@automerge/automerge';

type MyDoc = Record<string, unknown>;

function createAndChange() {
  let doc = Automerge.init();
  doc = Automerge.change<MyDoc>(doc, d => (d.hello = "from automerge"));
  return JSON.stringify(doc);
}

export default function App() {
  const result = createAndChange();
  return (
    <View style={styles.container}>
      <Text>Open up App.tsx to start working on your app!</Text>
      <StatusBar style="auto" />
      <Text>Result: {result}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#fff',
    alignItems: 'center',
    justifyContent: 'center',
  },
});
