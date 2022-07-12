import stripeX from 'stripe';
import { Elements, PaymentElement, useElements, useStripe } from "@stripe/react-stripe-js";
import { loadStripe, Stripe, StripeElements } from "@stripe/stripe-js";
import { RefObject, useEffect, useRef, useState } from "react";
import { backendUrlPrefix } from "../config";
import React from 'react';

export default function Card() {
    return <>
        <p>You mine CardToken by using a credit card or a bank account (unlike Bitcoin that is mined by costly equipment).</p>
        <p>To mine an amount of CardToken corresponding to a certain amount of money, pay any amount of money
            to your account 
            first your account will be anonymously stored in our database and then you pay.
            After you paid, our system will initiate crypto transfer to your account.
        </p>
        <PaymentForm/>
    </>
}

function PaymentForm() {
    const [options, setOptions] = useState(null as unknown as object);
    const [stripePromise, setStripePromise] = useState(null as Promise<Stripe | null> | null);
    const [fiatAmount, setFiatAmount] = useState(0);
    const [showPayment, setShowPayment] = useState(false);
    const [showPaymentError, setShowPaymentError] = useState("");
    const [paymentIntentId, setPaymentIntentId] = useState("");
    const userAccountRef = useRef(null);
    const fiatAmountRef = useRef<HTMLInputElement>(null);
    useEffect(() => {
        async function doIt() {
            const stripePubkey = await (await fetch(backendUrlPrefix + "/stripe-pubkey")).text(); // TODO: Fetch it only once.
            const fiatAmount = fiatAmountRef.current?.value as unknown as number * 100; // FIXME
            const res = await (await fetch(`${backendUrlPrefix}/create-payment-intent?fiat_amount=${fiatAmount}`)).json(); // FIXME
            if (res.error) {
                setShowPaymentError(res.error.message);
                setShowPayment(false);
            } else {
                const clientSecret: string = res["client_secret"];
                const paymentIntentId: string = res["id"];
                const stripePromise_: Promise<Stripe | null> = loadStripe(stripePubkey);

                setOptions({
                    clientSecret,
                    appearance: {},
                });
                setStripePromise(stripePromise_);
                setPaymentIntentId(paymentIntentId);
                setShowPayment(true);
            }
        }
        doIt();
    }, [fiatAmount]);

    return (
        <>
            <p>
                <label htmlFor="userAccount">Your crypto account:</label> {" "}
                <input type="text" id="userAccount" ref={userAccountRef}/> {" "}
                <label htmlFor="fiatAmount">Investment, in USD:</label> {" "}
                <input type="number" id="fiatAmount" ref={fiatAmountRef}
                    onChange={e => setFiatAmount(e.target.value as unknown as number)}/> {/* FIXME */}
            </p>
            {showPayment && <Elements stripe={stripePromise} options={options}>
                <PaymentFormContent paymentIntentId={paymentIntentId}/>
            </Elements>}
            {!showPayment && <p>{showPaymentError}</p>}
        </>
    );
}

var stripePaymentMethodHandler = function (result) {
    if (result.error) {
        // TODO: Show error in payment form
    } else {
      // Otherwise send paymentIntent.id to your server
        fetch('/confirmPayment', {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({
              payment_intent_id: result.paymentIntent.id,
            })
        }).then(function (res) {
            return res.json();
        }).then(function (paymentResponse) {
            // Handle server response (see Step 7)
            // TODO: handleServerResponse(paymentResponse);
        });
    }
};
  
function PaymentFormContent(props) {
    const stripe = useStripe() as Stripe;
    const elements = useElements() as StripeElements;

    async function submitHandler(event) {
        event.preventDefault();
      
        const stripePubkey = await (await fetch(backendUrlPrefix + "/stripe-pubkey")).text(); // TODO: Fetch it only once.
        const stripe = require('stripe')(stripePubkey);
        // const stripe = (await loadStripe(stripePubkey)) as Stripe;
        const pi = await stripe.paymentIntents.retrieve(props.paymentIntentId);
        alert(pi)
        pi.updatePaymentIntent({
           elements,
           params: {
             payment_method_data: {
               billing_details: { }
             },
             shipping: { }
           }
        }).then(function (result) {
            alert(111)
          stripePaymentMethodHandler(result)
        });
        alert(222)
    }

    return (
        <form onSubmit={submitHandler}>
            <PaymentElement />
            <p><button>Invest</button></p>
        </form>
    );
}

// async function initiatePayment() {
//     const userAddress = document.getElementById('userAccount');
//     await doInitiatePayment(userAddress);
// }
