# 🦀 RustStripe - Stripe Payment Integration in Rust

![RustStripe Banner](assets/rust-stripe-integration.png)

**Learning project**: Complete Stripe integration with Rust and Axum framework. Implements e-commerce shopping cart, recurring subscriptions, and payment method storage with real payment flows.

## 🎯 Exercices Implémentés

### 1. Gestion de Panier et Paiement E-commerce
- ✅ Catalogue de casquettes avec gestion des stocks
- ✅ Panier utilisateur
- ✅ Checkout avec Stripe PaymentIntent
- ✅ Confirmation/échec de paiement via webhooks
- ✅ Mise à jour automatique des stocks
- ✅ Historique complet des commandes

### 2. Abonnements Récurrents
- ✅ 3 formules d'abonnement (Normal 10€, Supplément 15€, Complet 20€)
- ✅ Création d'abonnement avec Stripe Subscriptions
- ✅ Notification par email (console log) lors des prélèvements
- ✅ Gestion des échecs de paiement (3 tentatives sur 3 jours)
- ✅ Suspension automatique après 3 échecs
- ✅ Notification d'expiration de carte bancaire

### 3. Sauvegarde de Moyens de Paiement
- ✅ Enregistrement de carte avec SetupIntent
- ✅ Liste des moyens de paiement avec identification
- ✅ Paiement avec carte sauvegardée
- ✅ Suppression de moyen de paiement
- ✅ Notification d'expiration de carte

## 🚀 Installation et Configuration

### Prérequis
- Rust 1.70+ ([installer Rust](https://rustup.rs/))
- Compte Stripe ([créer un compte](https://dashboard.stripe.com/register))
- Stripe CLI pour les webhooks ([installer Stripe CLI](https://stripe.com/docs/stripe-cli))

### Configuration

1. **Cloner et configurer le projet**
```powershell
cd d:\RustStripe
cp .env.example .env
```

2. **Configurer les clés Stripe dans `.env`**
```env
STRIPE_SECRET_KEY=sk_test_votre_cle_secrete
STRIPE_WEBHOOK_SECRET=whsec_votre_webhook_secret
BASE_URL=http://localhost:3000
```

Pour obtenir vos clés:
- Clé secrète: https://dashboard.stripe.com/test/apikeys
- Webhook secret: généré par Stripe CLI (voir section webhooks)

3. **Compiler et lancer le serveur**
```powershell
cargo build
cargo run
```

Le serveur démarre sur `http://localhost:3000`

## 🔗 Endpoints API

### Exercice 1: Panier & Paiement

#### Ajouter au panier
```powershell
curl -X POST http://localhost:3000/api/cart/add `
  -H "Content-Type: application/json" `
  -d '{
    "user_id": "user_123",
    "product_id": "cap_001",
    "quantity": 2
  }'
```

#### Voir le panier
```powershell
curl "http://localhost:3000/api/cart/view?user_id=user_123"
```

#### Passer commande (checkout)
```powershell
curl -X POST http://localhost:3000/api/cart/checkout `
  -H "Content-Type: application/json" `
  -d '{"user_id": "user_123"}'
```

#### Voir une commande
```powershell
curl http://localhost:3000/api/orders/{order_id}
```

#### Historique des commandes
```powershell
curl "http://localhost:3000/api/orders?user_id=user_123"
```

### Exercice 2: Abonnements

#### Créer un abonnement
```powershell
curl -X POST http://localhost:3000/api/subscriptions/create `
  -H "Content-Type: application/json" `
  -d '{
    "user_id": "user_123",
    "plan_id": "plan_normal",
    "email": "user@example.com"
  }'
```

Plans disponibles: `plan_normal` (10€), `plan_supplement` (15€), `plan_complet` (20€)

#### Voir un abonnement
```powershell
curl http://localhost:3000/api/subscriptions/{subscription_id}
```

#### Annuler un abonnement
```powershell
curl -X POST http://localhost:3000/api/subscriptions/{subscription_id}/cancel
```

### Exercice 3: Moyens de Paiement

#### Configurer un nouveau moyen de paiement
```powershell
curl -X POST http://localhost:3000/api/payment-methods/setup `
  -H "Content-Type: application/json" `
  -d '{"user_id": "user_123"}'
```

#### Lister les moyens de paiement
```powershell
curl "http://localhost:3000/api/payment-methods/list?user_id=user_123"
```

#### Payer avec une carte sauvegardée
```powershell
curl -X POST http://localhost:3000/api/payment-methods/pay `
  -H "Content-Type: application/json" `
  -d '{
    "user_id": "user_123",
    "payment_method_id": "pm_xxx",
    "amount": 5000,
    "description": "Achat rapide"
  }'
```

#### Supprimer un moyen de paiement
```powershell
curl -X POST http://localhost:3000/api/payment-methods/{pm_id}/delete
```

## 🔔 Configuration des Webhooks

Les webhooks Stripe permettent de recevoir les notifications en temps réel (paiement réussi, échec, etc.)

### Avec Stripe CLI (développement)

1. **Connecter Stripe CLI**
```powershell
stripe login
```

2. **Forwarding des webhooks**
```powershell
stripe listen --forward-to localhost:3000/webhooks/stripe
```

3. Copier le webhook secret (`whsec_...`) dans votre `.env`

### Événements gérés

- `payment_intent.succeeded` - Paiement réussi (mise à jour commande + stocks)
- `payment_intent.payment_failed` - Paiement échoué
- `setup_intent.succeeded` - Carte enregistrée avec succès
- `invoice.payment_succeeded` - Prélèvement abonnement réussi
- `invoice.payment_failed` - Échec prélèvement (réessai automatique)
- `customer.source.expiring` - Carte expire bientôt

## 🧪 Tests avec Stripe

### Cartes de test Stripe

```
Succès:           4242 4242 4242 4242
Échec:            4000 0000 0000 0002
3D Secure requis: 4000 0025 0000 3155
Fonds insuffisants: 4000 0000 0000 9995
```

Date d'expiration: n'importe quelle date future
CVC: n'importe quel 3 chiffres

### Scénario de test complet

1. **Test panier & paiement**
```powershell
# Ajouter des articles
curl -X POST http://localhost:3000/api/cart/add -H "Content-Type: application/json" -d '{"user_id":"user_123","product_id":"cap_001","quantity":1}'

# Voir le panier
curl "http://localhost:3000/api/cart/view?user_id=user_123"

# Checkout
curl -X POST http://localhost:3000/api/cart/checkout -H "Content-Type: application/json" -d '{"user_id":"user_123"}'

# Utiliser le client_secret retourné pour confirmer le paiement avec Stripe.js
# Le webhook mettra à jour automatiquement la commande
```

2. **Test abonnement**
```powershell
# Créer un abonnement
curl -X POST http://localhost:3000/api/subscriptions/create -H "Content-Type: application/json" -d '{"user_id":"user_456","plan_id":"plan_normal","email":"test@example.com"}'

# Observer les logs: notification de prélèvement mensuel via webhooks
```

3. **Test carte sauvegardée**
```powershell
# Setup
curl -X POST http://localhost:3000/api/payment-methods/setup -H "Content-Type: application/json" -d '{"user_id":"user_789"}'

# Utiliser le client_secret pour enregistrer la carte avec Stripe Elements
# Le webhook confirmera l'enregistrement

# Payer avec la carte
curl -X POST http://localhost:3000/api/payment-methods/pay -H "Content-Type: application/json" -d '{"user_id":"user_789","payment_method_id":"pm_xxx","amount":3000,"description":"Test"}'
```

## 📦 Structure du Projet

```
src/
├── main.rs              # Point d'entrée, configuration serveur
├── config.rs            # Configuration (variables d'environnement)
├── state.rs             # État partagé de l'application
├── models.rs            # Structures de données
├── routes/
│   ├── cart.rs          # Routes panier & paiement
│   ├── subscriptions.rs # Routes abonnements
│   ├── payment_methods.rs # Routes moyens de paiement
│   └── webhooks.rs      # Handler webhooks Stripe
└── services/
    └── stripe_service.rs # Intégration API Stripe
```

## 🎓 Concepts Rust/Axum Utilisés

- **Axum** - Framework web moderne basé sur Tower et Hyper
- **Tokio** - Runtime asynchrone
- **Stripe-rust** - Client officiel Stripe pour Rust
- **DashMap** - HashMap thread-safe pour le stockage en mémoire
- **Serde** - Sérialisation/désérialisation JSON
- **State management** - Partage d'état avec Arc
- **Error handling** - Gestion d'erreurs avec Result et Status codes

## 📚 Ressources

- [Documentation Stripe](https://stripe.com/docs)
- [Documentation Axum](https://docs.rs/axum/latest/axum/)
- [Stripe Testing](https://stripe.com/docs/testing)
- [Stripe Webhooks](https://stripe.com/docs/webhooks)

## ⚠️ Notes Importantes

- Ce projet utilise une base de données en mémoire (DashMap) pour la démonstration. En production, utilisez une vraie base de données (PostgreSQL, MongoDB, etc.)
- Les notifications "email" sont simulées via console logs
- Toujours vérifier la signature des webhooks en production
- Les clés API sont dans `.env` et ne doivent JAMAIS être commitées

## 🔐 Sécurité

En production, assurez-vous de:
- ✅ Vérifier les signatures webhooks avec `stripe_webhook_secret`
- ✅ Utiliser HTTPS
- ✅ Valider toutes les entrées utilisateur
- ✅ Ne jamais exposer les clés secrètes
- ✅ Implémenter l'authentification utilisateur
- ✅ Logger tous les événements sensibles

Bon apprentissage avec Rust et Stripe! 🦀💳
