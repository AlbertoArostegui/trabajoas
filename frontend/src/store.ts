// src/store.js or src/store.ts
import { createStore, Store as VuexStore } from 'vuex'

export type State = {
  user: null | { [key: string]: any }; // replace { [key: string]: any } with the actual type of your user
  is_logged_in: boolean;
}

export default createStore<State>({
  state: {
    user: null,
    is_logged_in: false
  },
  mutations: {
    setUser(state, user) {
      state.user = user
      state.is_logged_in = true
    },
    unsetUser(state, user) {
        state.user = user
        state.is_logged_in = false
    }
  },
  actions: {
    login({ commit }, user) {
      // Perform the login operation and get the user data
      commit('setUser', user)
    },
    logout({ commit }) {
      // Clear the user data
      commit('unsetUser', null)
    }
  },
  getters: {
    is_logged_in: state => !!state.user
  }
})