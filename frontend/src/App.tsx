import { BrowserRouter, NavLink, Route, Routes } from "react-router-dom";
import Bid from './Bid';
import Card from './Card';
import Withdraw from './Withdraw';
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
        <h1>Bid world token</h1>
        <BrowserRouter>
            <nav className="mainNav">
                <MyNavLink to={`/bid`}>Bid DOT token</MyNavLink> |{" "}
                {/*<MyNavLink to={`/card`}>Bid with a card or bank</MyNavLink> |{" "}*/}
                <MyNavLink to={`/withdraw`}>Withdraw earninigs</MyNavLink>
            </nav>
            <Routes>
                <Route path="/bid" element={<Bid/>} />
                <Route path="/card" element={<Card/>} />
                <Route path="/withdraw" element={<Withdraw/>} />
            </Routes>
        </BrowserRouter>
    </>
}   