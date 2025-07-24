import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import './App.css';
import {CodeExetutorInput} from './components/code-executor/CodeExetutorInput';
import { Auth } from './components/auth/Auth';

function App() {
  return (
    <div className="App">
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Navigate to="/auth" replace />} />
          <Route path="/code" element={<CodeExetutorInput/>} />
          <Route path="/auth" element={<Auth/>} />
        </Routes>
      </BrowserRouter>
    </div>
  );
}

export default App;
