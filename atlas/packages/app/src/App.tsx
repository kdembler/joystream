import React from 'react'
import { Provider } from 'react-redux'

import store from './store'
import { LayoutWithRouting } from './components'

export default function App() {
  return (
    <Provider store={store}>
      <LayoutWithRouting />
    </Provider>
  )
}
