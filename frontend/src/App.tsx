import { BrowserRouter, NavLink, Route, Routes } from "react-router-dom";
import Bid from './Bid';
import Card from './Card';
import Withdraw from './Withdraw';
import "./App.css";
import React, { ReactNode } from "react";
import { Login, Register } from './User';

function MyNavLink(props: {to: string, children: ReactNode}) {
    return (
        // TODO: className instead.
        <NavLink to={props.to}
            className={({ isActive }) => (isActive ? 'active' : 'inactive')}
        >{props.children}</NavLink>
    );
}

export function App() {
    return (
        <div style={{padding: '5px'}}>
            <h1>Bid CardToken</h1>
            <BrowserRouter>
                <nav className="mainNav">
                    <MyNavLink to={`/bid`}>Bid</MyNavLink>
                    <MyNavLink to={`/withdraw`}>Withdraw earninigs</MyNavLink>
                </nav>
                <Routes>
                    <Route path="/bid" element={<Bid/>} />
                    <Route path="/withdraw" element={<Withdraw/>} />
                    <Route path="/register" element={<Register/>} />
                    <Route path="/login" element={<Login/>} />
                </Routes>
            </BrowserRouter>
        </div>
    );
}   