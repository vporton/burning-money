import { Elements, PaymentElement } from "@stripe/react-stripe-js";
import { loadStripe, Stripe } from "@stripe/stripe-js";
import { useEffect, useRef, useState } from "react";
import { backendUrlPrefix } from "../config";

export default function Card() {
    const userAccount = useRef(null);
    return <>
        <p>You mine CardToken by using a credit card or a bank account (unlike Bitcoin that is mined by costly equipment).</p>
        <p>To mine an amount of CardToken corresponding to a certain amount of money, pay any amount of money
            to your account <input type="text" id="userAccount" ref={userAccount}/>
            first your account will be anonymously stored in our database and then you pay.
            After you paid, our system will initiate money transfer to your account.
            <strong>You must pay during 12 hours since clicking the button. If you pay late,
                your payment may be stalled in a crypto account with no one being able to get the money.</strong>
        </p>
        <PaymentForm/>
    </>
}

function PaymentForm(userAddress) {
    const [options, setOptions] = useState(null as unknown as object);
    const [stripePromise, setStripePromise] = useState(null as Promise<Stripe | null> | null);
    useEffect(() => {
        async function doIt() {
            const stripe_pubkey = await (await fetch(backendUrlPrefix + "/stripe-pubkey")).text();
            const res = await (await fetch(backendUrlPrefix + "/create-payment-intent?fiat_amount=10")).json(); // FIXME
            const client_secret: string = res["client_secret"];
            console.log("TTT", stripe_pubkey, client_secret);

            const stripePromise_: Promise<Stripe | null> = loadStripe(stripe_pubkey);

            setOptions({
                clientSecret: client_secret,
                appearance: {},
            });
            setStripePromise(stripePromise_);
        }
        doIt();
    }, []);

    return (
        <Elements stripe={stripePromise} options={options}>
            <form>
                <PaymentElement />
                <button>Submit</button>
            </form>
        </Elements>
    );
}


// async function initiatePayment() {
//     const userAddress = document.getElementById('userAccount');
//     await doInitiatePayment(userAddress);
// }
