import { Elements, PaymentElement, useElements, useStripe } from "@stripe/react-stripe-js";
import { loadStripe, PaymentIntent, Stripe, StripeElements } from "@stripe/stripe-js";
import { FormEvent, RefObject, useEffect, useRef, useState } from "react";
import { backendUrlPrefix } from "./config";
import React from 'react';
import { NavLink } from "react-router-dom";

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
    const [showPaymentError, setShowPaymentError] = useState("");
    const [paymentIntentId, setPaymentIntentId] = useState(""); // FIXME: It uses the first successul payment intent ID, rather than for the last entered fiat value.
    const [userAccount, setUserAccount] = useState("");
    const fiatAmountRef = useRef<HTMLInputElement>(null);
    const payButtonRef = useRef<HTMLButtonElement>(null);

    async function createPaymentForm() {
        (fiatAmountRef.current as HTMLInputElement).disabled = true;
        (payButtonRef.current as HTMLButtonElement).disabled = true;

        const stripePubkey = await (await fetch(backendUrlPrefix + "/stripe-pubkey")).text(); // TODO: Fetch it only once.
        const fiatAmount = fiatAmountRef.current?.value as unknown as number * 100; // FIXME
        const res = await (await fetch(`${backendUrlPrefix}/create-payment-intent?fiat_amount=${fiatAmount}`, {
            method: "POST",
            credentials: 'include',
        })).json(); // FIXME
        if (res.error) {
            setShowPaymentError(res.error.message);
            setShowPayment(false);
        } else {
            const clientSecret: string = res["client_secret"];
            const paymentIntentId: string = res["id"];
            const stripePromise_: Promise<Stripe | null> = loadStripe(stripePubkey, {
                betas: ['server_side_confirmation_beta_1'],
                apiVersion: '2020-08-27;server_side_confirmation_beta=v1',
            });

            setOptions({
                clientSecret,
                appearance: {},
            });
            setStripePromise(stripePromise_);
            setPaymentIntentId(paymentIntentId); // FIXME: It doesn't update after Submit.
            setShowPayment(true);
        }
    }

    return (
        <>
            <p>
                <label htmlFor="userAccount">Your crypto account:</label> {" "}
                <input type="text" id="userAccount" onChange={e => setUserAccount(e.target.value)}/> {" "}
                <label htmlFor="fiatAmount">Investment, in USD:</label> {" "}
                <input type="number" id="fiatAmount" ref={fiatAmountRef}
                    onChange={e => setFiatAmount(e.target.value as unknown as number)}/> {/* FIXME */}
                <button ref={payButtonRef} disabled={fiatAmount < 0.5} onClick={e => createPaymentForm()}>Pay</button>
            </p>
            {showPayment && <Elements stripe={stripePromise} options={options}>
                <PaymentFormContent paymentIntentId={paymentIntentId} userAccount={userAccount} bidDate={props.bidDate}/>
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
                    console.log('paymentResponse: ', paymentResponse)
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
