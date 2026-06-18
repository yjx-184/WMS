import { createBrowserRouter, Navigate } from 'react-router-dom';
import AppLayout from '../components/AppLayout';
import ProductList from '../pages/ProductList';
import WarehouseList from '../pages/WarehouseList';
import LocationList from '../pages/LocationList';
import InboundList from '../pages/InboundList';
import InboundForm from '../pages/InboundForm';
import InboundDetail from '../pages/InboundDetail';
import OutboundList from '../pages/OutboundList';
import OutboundForm from '../pages/OutboundForm';
import NotFound from '../pages/NotFound';

/* ------------------------------------------------------------------ */
/*  Placeholder page components — replaced with real pages in later   */
/*  tasks (T5.2.3, T6.2.1).                                            */
/* ------------------------------------------------------------------ */

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
      { path: 'outbounds/:id', element: <OutboundForm /> },

      /* inventory */
      { path: 'inventory', element: <InventoryQuery /> },

      /* 404 catch-all */
      { path: '*', element: <NotFound /> },
    ],
  },
]);
