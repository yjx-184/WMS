import { createBrowserRouter, Navigate } from 'react-router-dom';
import AppLayout from '../components/AppLayout';
import ProductList from '../pages/ProductList';
import NotFound from '../pages/NotFound';

/* ------------------------------------------------------------------ */
/*  Placeholder page components — replaced with real pages in later   */
/*  tasks (T2.2.1, T3.2.1, T4.2.x, T5.2.x, T6.2.1).                  */
/* ------------------------------------------------------------------ */

function WarehouseList() {
  return <div>WarehouseList placeholder</div>;
}
function LocationList() {
  return <div>LocationList placeholder</div>;
}
function InboundList() {
  return <div>InboundList placeholder</div>;
}
function InboundForm() {
  return <div>InboundForm placeholder</div>;
}
function InboundDetail() {
  return <div>InboundDetail placeholder</div>;
}
function OutboundList() {
  return <div>OutboundList placeholder</div>;
}
function OutboundForm() {
  return <div>OutboundForm placeholder</div>;
}
function OutboundDetail() {
  return <div>OutboundDetail placeholder</div>;
}
function InventoryQuery() {
  return <div>InventoryQuery placeholder</div>;
}

/* ------------------------------------------------------------------ */
/*  Route tree                                                         */
/* ------------------------------------------------------------------ */

export const router = createBrowserRouter([
  {
    path: '/',
    element: <AppLayout />,
    children: [
      /* default redirect */
      { index: true, element: <Navigate to="/products" replace /> },

      /* products */
      { path: 'products', element: <ProductList /> },

      /* warehouses */
      { path: 'warehouses', element: <WarehouseList /> },
      { path: 'warehouses/:id/locations', element: <LocationList /> },

      /* inbound */
      { path: 'inbounds', element: <InboundList /> },
      { path: 'inbounds/new', element: <InboundForm /> },
      { path: 'inbounds/:id', element: <InboundDetail /> },

      /* outbound */
      { path: 'outbounds', element: <OutboundList /> },
      { path: 'outbounds/new', element: <OutboundForm /> },
      { path: 'outbounds/:id', element: <OutboundDetail /> },

      /* inventory */
      { path: 'inventory', element: <InventoryQuery /> },

      /* 404 catch-all */
      { path: '*', element: <NotFound /> },
    ],
  },
]);
