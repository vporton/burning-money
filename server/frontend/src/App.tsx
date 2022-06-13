import { BrowserRouter, NavLink, Route, Routes } from "react-router-dom";
import Bid from './Bid';
import Card from './Card';
import "./App.css";

function MyNavLink(props) {
    return (
        // TODO: className instead.
        // FIXME: isActive does not work as expected.
        <NavLink to={props.to}
            className={({ isActive }) => (isActive ? 'active' : 'inactive')}
        >{props.children}</NavLink>
    );
}

export function App() {
    return <>
        <BrowserRouter>
            <nav className="mainNav">
                <MyNavLink to={`/bid`}>Bid</MyNavLink> |{" "}
                <MyNavLink to={`/card`}>Pay with a card</MyNavLink>
            </nav>
            <Routes>
                <Route path="/bid" element={<Bid/>} />
                <Route path="/card" element={<Card/>} />
            </Routes>
        </BrowserRouter>
    </>
}   