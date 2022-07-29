import { Elements, PaymentElement, useElements, useStripe } from "@stripe/react-stripe-js";
import { loadStripe, PaymentIntent, Stripe, StripeElements } from "@stripe/stripe-js";
import { FormEvent, RefObject, useEffect, useRef, useState } from "react";
import { backendUrlPrefix } from "./config";
import React from 'react';
import { NavLink } from "react-router-dom";
import { EthAddress } from "./components/EthAddress";

export default function Card(props: { bidDate: Date }) {
    const [user, setUser] = useState<string | null>(null);
    fetch(backendUrlPrefix + "/identity", {credentials: 'include'})
        .then(u => u.json())
        .then(u => {
            setUser(u.id);
        });
    function logout() {
        window.fetch(backendUrlPrefix + "/logout", {
            method: "POST",
            credentials: 'include',
        })
            .then(() => setUser(null)); // TODO: Handle errors.
    }

    return <>
        {user === null ? <><NavLink to={'/login'}>Login</NavLink> <NavLink to={'/register'}>Register</NavLink></> : <a href='#' onClick={logout}>Logout</a>}
        <p>You mine CardToken by using a credit card or a bank account (unlike Bitcoin that is mined by costly equipment).</p>
        <p>To mine an amount of CardToken corresponding to a certain amount of money, pay any amount of money.</p>
        {user !== null ? <PaymentForm bidDate={props.bidDate}/> : ""}
    </>
}

// https://stripe.com/docs/payments/finalize-payments-on-the-server

function PaymentForm(props: { bidDate: Date }) {
    const [options, setOptions] = useState(null as unknown as object);
    const [stripePromise, setStripePromise] = useState(null as Promise<Stripe | null> | null);
    const [fiatAmount, setFiatAmount] = useState(0);
    const [showPayment, setShowPayment] = useState(false);
    const [showingPayment, setShowingPayment] = useState(false);
    const [showPaymentError, setShowPaymentError] = useState("");
    const [paymentIntentId, setPaymentIntentId] = useState("");
    const [userAccount, setUserAccount] = useState("");
    const [ethAddrValid, setEthAddrValid] = useState(false);
    const fiatAmountRef = useRef<HTMLInputElement>(null);
    const payButtonRef = useRef<HTMLButtonElement>(null);

    function handleAccountsChanged(accounts: any) {
        if(accounts[0]) {
            setUserAccount(accounts[0]);
        }
    }

    (window as any).ethereum
        .request({ method: 'eth_accounts' })
        .then(handleAccountsChanged)
        .catch((err: any) => {
            console.error(err);
        });
    (window as any).ethereum.on('accountsChanged', handleAccountsChanged);


    async function createPaymentIntent() {
        const res = await (await fetch(`${backendUrlPrefix}/create-payment-intent?fiat_amount=${fiatAmount}`, {
            method: "POST",
            credentials: 'include',
        })).json();

        if (res.error) {
            setShowPaymentError(res.error.message);
            setShowPayment(false);
        } else {
            const clientSecret: string = res["client_secret"];
            const paymentIntentId: string = res["id"];

            setOptions({
                clientSecret,
                appearance: {},
            });
            setPaymentIntentId(paymentIntentId);
        }
    }

    async function doShowPayment() {
        setShowingPayment(true);
        await createPaymentIntent();
        setShowPayment(true);
    }

    useEffect(() => {
        fetch(backendUrlPrefix + "/stripe-pubkey")
            .then(res => res.text())
            .then(stripePubkey => {
                const stripePromise_: Promise<Stripe | null> = loadStripe(stripePubkey, {
                    betas: ['server_side_confirmation_beta_1'],
                    apiVersion: '2020-08-27;server_side_confirmation_beta=v1',
                });
                setStripePromise(stripePromise_);
            });
    }, []);

    function onPayClicked() {
        setPaymentIntentId("");
    }

    function setFiatAmountFromInput(input: HTMLInputElement) {
        setFiatAmount(Math.floor(0.5 + (Number(input.value) * 100)))
    }

    return (
        <>
            <p>
                <label htmlFor="userAccount">Your crypto account:</label> {" "}
                <EthAddress id="userAccount" value={userAccount} onChange={(e: any) => setUserAccount(e.target.value)} onValid={setEthAddrValid}/> {" "}
                <label htmlFor="fiatAmount">Investment, in USD:</label> {" "}
                <input type="number" id="fiatAmount" ref={fiatAmountRef}
                    onChange={e => setFiatAmountFromInput(e.target)} disabled={showingPayment}/> {" "}
                <button ref={payButtonRef} disabled={!ethAddrValid || fiatAmount < 0.5 || !stripePromise || showingPayment} onClick={e => doShowPayment()}
                >Next &gt;&gt;</button>
                {showingPayment ?
                    <>
                        {" "}
                        <button onClick={() => { setShowingPayment(false); setShowPayment(false); createPaymentIntent(); }}>&lt;&lt; Back</button>
                    </>
                    : ""}
            </p>
            {showPayment && <Elements stripe={stripePromise} options={options}>
                <PaymentFormContent paymentIntentId={paymentIntentId} userAccount={userAccount} bidDate={props.bidDate} onPayClicked={onPayClicked}
                onPaid={() => {setShowingPayment(false); setShowPayment(false);}}/>
            </Elements>}
            {!showPayment && <p>{showPaymentError}</p>}
        </>
    );
}

function PaymentFormContent(props: any) { // TODO: `any`
    const stripe = useStripe() as Stripe;
    const elements = useElements() as StripeElements;

    async function submitHandler(event: FormEvent<HTMLFormElement>) {
        event.preventDefault();
      
        const handleServerResponse = async (response: any) => {
            if (response.error) {
                alert(response.error); // FIXME
            } else if (response.requires_action) {
                // Use Stripe.js to handle the required next action
                const {
                    error: errorAction,
                    paymentIntent
                } = await (stripe as any).handleNextAction({
                    clientSecret: response.payment_intent_client_secret
                });

                if (errorAction) {
                    alert(errorAction); // TODO
                } else {
                    alert("Success."); // FIXME
                }
            } else {
                props.onPaid();
                alert("You've paid."); // TODO
            }
          }
        
        const stripePaymentMethodHandler = function (result: any) {
            if (result.error) {
                alert(result.error); // TODO
            } else {
                // Otherwise send paymentIntent.id to your server
                let data = [];
                data.push('payment_intent_id=' + encodeURIComponent(result.paymentIntent.id));
                data.push('crypto_account=' + encodeURIComponent(props.userAccount));
                data.push('bid_date=' + encodeURIComponent(props.bidDate.toISOString()));
                fetch(backendUrlPrefix + '/confirm-payment', {
                    method: 'POST',
                    credentials: 'include',
                    mode: 'cors',
                    headers: {'Content-Type': 'application/x-www-form-urlencoded'},
                    body: data.join('&'),
                }).then(function (res) {
                    return res.json();
                }).then(function (paymentResponse) {
                    handleServerResponse(paymentResponse);
                });
            }
        };

        (stripe as any).updatePaymentIntent({
            elements, // elements instance
            params: {
            //   payment_method_data: {
            //     billing_details: { ... }
            //   },
            //   shipping: { ... }
            }
        }).then(function (result: any) {
           stripePaymentMethodHandler(result)
        });
    }

    return (
        <form onSubmit={e => submitHandler(e)}> {/* FIXME: async */}
            <PaymentElement />
            <p><button>Invest</button></p> {/* TODO: Disable button for no accidental repeated clicking. */}
        </form>
    );
}

// async function initiatePayment() {
//     const userAddress = document.getElementById('userAccount');
//     await doInitiatePayment(userAddress);
// }
